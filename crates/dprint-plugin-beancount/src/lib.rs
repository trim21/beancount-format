#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

use beancount_formatter::configuration::Configuration;
use beancount_formatter::configuration::NewLineKind;
use beancount_formatter::format as format_beancount;
use dprint_core::configuration::ConfigKeyMap;
use dprint_core::configuration::GlobalConfiguration;
use dprint_core::configuration::NewLineKind as DprintNewLineKind;
use dprint_core::generate_plugin_code;
use dprint_core::plugins::CheckConfigUpdatesMessage;
use dprint_core::plugins::ConfigChange;
use dprint_core::plugins::FileMatchingInfo;
use dprint_core::plugins::FormatResult;
use dprint_core::plugins::PluginInfo;
use dprint_core::plugins::PluginResolveConfigurationResult;
use dprint_core::plugins::SyncFormatRequest;
use dprint_core::plugins::SyncHostFormatRequest;
use dprint_core::plugins::SyncPluginHandler;
use std::alloc::Layout;
use std::mem::{align_of, size_of};
use std::ptr;

struct BeancountPluginHandler;

impl SyncPluginHandler<Configuration> for BeancountPluginHandler {
  fn resolve_config(
    &mut self,
    config: ConfigKeyMap,
    global_config: &GlobalConfiguration,
  ) -> PluginResolveConfigurationResult<Configuration> {
    let config = resolve_config_dprint(config, global_config);
    PluginResolveConfigurationResult {
      config: config.config,
      diagnostics: config.diagnostics,
      file_matching: FileMatchingInfo {
        file_extensions: vec!["beancount".to_string(), "bean".to_string()],
        file_names: vec![],
      },
    }
  }

  fn check_config_updates(&self, _message: CheckConfigUpdatesMessage) -> Result<Vec<ConfigChange>, anyhow::Error> {
    Ok(Vec::new())
  }

  fn plugin_info(&mut self) -> PluginInfo {
    let version = env!("CARGO_PKG_VERSION").to_string();
    PluginInfo {
      name: "@trim21/dprint-plugin-beancount".to_string(),
      version: version.clone(),
      config_key: "beancount".to_string(),
      help_url: "https://github.com/trim21/beancount-format".to_string(),
      config_schema_url: format!(
        "https://cdn.jsdelivr.net/gh/trim21/beancount-format@gh-pages/dprint-plugin-beancount/{}/schema.json",
        version
      ),
      update_url: Some(
        "https://cdn.jsdelivr.net/gh/trim21/beancount-format@gh-pages/dprint-plugin-beancount/latest.json".to_string(),
      ),
    }
  }

  fn license_text(&mut self) -> String {
    std::str::from_utf8(include_bytes!("../../../LICENSE")).unwrap().into()
  }

  fn format(
    &mut self,
    request: SyncFormatRequest<Configuration>,
    _format_with_host: impl FnMut(SyncHostFormatRequest) -> FormatResult,
  ) -> FormatResult {
    let file_text = String::from_utf8(request.file_bytes)?;
    if file_text.trim().is_empty() {
      return Ok(Some(Vec::new()));
    }
    let formatted = format_beancount(request.file_path.to_str(), &file_text, &request.config)?;

    if formatted == file_text {
      Ok(None)
    } else {
      Ok(Some(formatted.into_bytes()))
    }
  }
}

generate_plugin_code!(BeancountPluginHandler, BeancountPluginHandler);

fn resolve_config_dprint(
  config: ConfigKeyMap,
  global_config: &GlobalConfiguration,
) -> dprint_core::configuration::ResolveConfigurationResult<Configuration> {
  use dprint_core::configuration::RECOMMENDED_GLOBAL_CONFIGURATION;
  use dprint_core::configuration::ResolveConfigurationResult;
  use dprint_core::configuration::get_unknown_property_diagnostics;
  use dprint_core::configuration::get_value;

  let mut diagnostics = Vec::new();
  let mut config = config;

  let resolved_config = Configuration {
    line_width: get_value(
      &mut config,
      "line_width",
      global_config
        .line_width
        .unwrap_or(RECOMMENDED_GLOBAL_CONFIGURATION.line_width),
      &mut diagnostics,
    ),
    indent_width: get_value(
      &mut config,
      "indent_width",
      global_config.indent_width.unwrap_or(2),
      &mut diagnostics,
    ),
    new_line: map_new_line_kind(get_value(
      &mut config,
      "new_line",
      global_config
        .new_line_kind
        .unwrap_or(RECOMMENDED_GLOBAL_CONFIGURATION.new_line_kind),
      &mut diagnostics,
    )),
    ..Configuration::default()
  };

  diagnostics.extend(get_unknown_property_diagnostics(config));

  ResolveConfigurationResult {
    config: resolved_config,
    diagnostics,
  }
}

fn map_new_line_kind(value: DprintNewLineKind) -> NewLineKind {
  match value {
    DprintNewLineKind::LineFeed => NewLineKind::LF,
    DprintNewLineKind::CarriageReturnLineFeed => NewLineKind::CRLF,
    _ => NewLineKind::LF,
  }
}

#[global_allocator]
static ALLOCATOR: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

// C `malloc` must return a pointer suitably aligned for any object type.
// We use a conservative alignment that is at least pointer-sized and typically 16.
const MALLOC_ALIGN: usize = {
  let a = align_of::<u128>();
  if a > align_of::<usize>() {
    a
  } else {
    align_of::<usize>()
  }
};

const fn align_up(value: usize, align: usize) -> usize {
  // `align` is expected to be a power of two.
  (value + (align - 1)) & !(align - 1)
}

// Pad the header so that (raw_ptr + HEADER_SIZE) is still MALLOC_ALIGN-aligned.
const HEADER_SIZE: usize = align_up(size_of::<usize>(), MALLOC_ALIGN);
const HEADER_ALIGN: usize = MALLOC_ALIGN;

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
  if size == 0 {
    return ptr::null_mut();
  }

  let total_size = match size.checked_add(HEADER_SIZE) {
    Some(v) => v,
    None => return ptr::null_mut(),
  };

  let layout = match Layout::from_size_align(total_size, HEADER_ALIGN) {
    Ok(l) => l,
    Err(_) => return ptr::null_mut(),
  };

  unsafe {
    let raw_ptr = std::alloc::alloc(layout);
    if raw_ptr.is_null() {
      return ptr::null_mut();
    }

    (raw_ptr as *mut usize).write(size);
    raw_ptr.add(HEADER_SIZE)
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn calloc(nitems: usize, size: usize) -> *mut u8 {
  let total = match nitems.checked_mul(size) {
    Some(v) => v,
    None => return ptr::null_mut(),
  };

  let ptr = malloc(total);
  if ptr.is_null() {
    return ptr;
  }

  unsafe {
    ptr::write_bytes(ptr, 0, total);
  }

  ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn free(ptr: *mut u8) {
  if ptr.is_null() {
    return;
  }

  unsafe {
    let header_ptr = ptr.sub(HEADER_SIZE);
    let size = header_ptr.cast::<usize>().read();
    let total_size = match size.checked_add(HEADER_SIZE) {
      Some(v) => v,
      None => return,
    };

    if let Ok(layout) = Layout::from_size_align(total_size, HEADER_ALIGN) {
      std::alloc::dealloc(header_ptr, layout);
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn realloc(ptr: *mut u8, size: usize) -> *mut u8 {
  if ptr.is_null() {
    return malloc(size);
  }

  if size == 0 {
    free(ptr);
    return ptr::null_mut();
  }

  unsafe {
    let header_ptr = ptr.sub(HEADER_SIZE);
    let old_size = header_ptr.cast::<usize>().read();
    let old_total = match old_size.checked_add(HEADER_SIZE) {
      Some(v) => v,
      None => return ptr::null_mut(),
    };
    let new_total = match size.checked_add(HEADER_SIZE) {
      Some(v) => v,
      None => return ptr::null_mut(),
    };

    let old_layout = match Layout::from_size_align(old_total, HEADER_ALIGN) {
      Ok(l) => l,
      Err(_) => return ptr::null_mut(),
    };

    let new_header_ptr = std::alloc::realloc(header_ptr, old_layout, new_total);
    if new_header_ptr.is_null() {
      return ptr::null_mut();
    }

    new_header_ptr.cast::<usize>().write(size);
    new_header_ptr.add(HEADER_SIZE)
  }
}
