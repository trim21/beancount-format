use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Result};
use beancount_formatter::configuration::Configuration;
use beancount_formatter::format;
use clap::Parser;
use toml::de::Error as TomlError;

const SUPPORTED_EXTENSIONS: &[&str] = &["beancount", "bean"];

/// Simple CLI to format beancount files.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
  /// Paths to beancount files or directories containing them.
  #[arg(value_name = "PATH", num_args = 1..)]
  input: Vec<PathBuf>,
  /// Write changes back to the file instead of printing to stdout.
  #[arg(short, long)]
  in_place: bool,
}

fn main() -> Result<()> {
  let args = Cli::parse();
  let config = load_configuration(&args.input)?;
  let files = collect_files(&args.input)?;
  let mut any_changed = false;

  for path in files {
    let content = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let path_str = path.to_string_lossy();
    let formatted = format(Some(&path_str), &content, &config)?;

    if formatted == content {
      if !args.in_place {
        print!("{}", content);
      }
    } else if args.in_place {
      fs::write(&path, &formatted).with_context(|| format!("Failed to write {}", path.display()))?;
    } else {
      print!("{}", formatted);
    }

    if formatted != content {
      any_changed = true;
    }
  }

  if any_changed {
    process::exit(1);
  }

  Ok(())
}

fn load_configuration(inputs: &[PathBuf]) -> Result<Configuration> {
  let mut config = Configuration::default();

  if let Some(pyproject_path) = find_pyproject(inputs) {
    let content =
      fs::read_to_string(&pyproject_path).with_context(|| format!("Failed to read {}", pyproject_path.display()))?;

    let parsed = parse_pyproject(&content).with_context(|| format!("Failed to parse {}", pyproject_path.display()))?;

    if let Some(tool) = parsed.tool
      && let Some(cfg) = tool.beancount_formatter
    {
      cfg.apply(&mut config);
    }
  }

  Ok(config)
}

fn collect_files(inputs: &[PathBuf]) -> Result<Vec<PathBuf>> {
  let mut files = Vec::new();

  for input in inputs {
    collect_path(input, &mut files)?;
  }

  if files.is_empty() {
    anyhow::bail!("No .beancount or .bean files found in the provided paths");
  }

  Ok(files)
}

fn find_pyproject(inputs: &[PathBuf]) -> Option<PathBuf> {
  let mut roots = Vec::new();

  if let Ok(cwd) = env::current_dir() {
    roots.push(cwd);
  }

  for input in inputs {
    let start = match fs::metadata(input) {
      Ok(md) if md.is_file() => input.parent().map(|p| p.to_path_buf()),
      Ok(md) if md.is_dir() => Some(input.to_path_buf()),
      _ => None,
    };

    if let Some(dir) = start {
      roots.push(dir);
    }
  }

  for mut dir in roots {
    loop {
      let candidate = dir.join("pyproject.toml");
      if candidate.is_file() {
        return Some(candidate);
      }

      if !dir.pop() {
        break;
      }
    }
  }

  None
}

fn collect_path(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
  let metadata = fs::metadata(path).with_context(|| format!("Failed to access {}", path.display()))?;

  if metadata.is_dir() {
    collect_dir(path, files)?;
  } else if metadata.is_file() && is_supported_file(path) {
    files.push(path.to_path_buf());
  }

  Ok(())
}

fn collect_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
  let mut entries = fs::read_dir(dir)
    .with_context(|| format!("Failed to read directory {}", dir.display()))?
    .collect::<Result<Vec<_>, _>>()?;

  entries.sort_by_key(|a| a.path());

  for entry in entries {
    let path = entry.path();
    let metadata = entry
      .metadata()
      .with_context(|| format!("Failed to access {}", path.display()))?;

    if metadata.is_dir() {
      collect_dir(&path, files)?;
    } else if metadata.is_file() && is_supported_file(&path) {
      files.push(path);
    }
  }

  Ok(())
}

fn is_supported_file(path: &Path) -> bool {
  path
    .extension()
    .and_then(|ext| ext.to_str())
    .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
    .unwrap_or(false)
}

#[derive(Debug, Default, serde::Deserialize)]
struct Pyproject {
  tool: Option<ToolSection>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct ToolSection {
  #[serde(rename = "beancount-formatter")]
  beancount_formatter: Option<PartialConfiguration>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct PartialConfiguration {
  line_width: Option<u32>,
  indent_width: Option<u8>,
  new_line_kind: Option<beancount_formatter::configuration::NewLineKind>,
  prefix_width: Option<usize>,
  num_width: Option<usize>,
  currency_column: Option<usize>,
  account_amount_spacing: Option<usize>,
  number_currency_spacing: Option<usize>,
}

impl PartialConfiguration {
  fn apply(self, config: &mut Configuration) {
    config.line_width = self.line_width.unwrap_or(config.line_width);
    config.indent_width = self.indent_width.unwrap_or(config.indent_width);
    config.new_line_kind = self.new_line_kind.unwrap_or(config.new_line_kind);
    config.prefix_width = self.prefix_width.or(config.prefix_width);
    config.num_width = self.num_width.or(config.num_width);
    config.currency_column = self.currency_column.or(config.currency_column);
    config.account_amount_spacing = self.account_amount_spacing.or(config.account_amount_spacing);
    config.number_currency_spacing = self.number_currency_spacing.or(config.number_currency_spacing);
  }
}

fn parse_pyproject(content: &str) -> Result<Pyproject, TomlError> {
  toml::from_str(content)
}
