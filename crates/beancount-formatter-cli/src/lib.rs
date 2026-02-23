use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use beancount_formatter::configuration::{
  Configuration, NewLineKind, PartialConfiguration as CorePartialConfiguration,
};
use beancount_formatter::format;
use clap::Parser;
use toml::de::Error as TomlError;

const SUPPORTED_EXTENSIONS: &[&str] = &["beancount", "bean"];

/// Simple CLI to format beancount files.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
  /// Paths to beancount files or directories containing them.
  #[arg(value_name = "PATH", num_args = 1..)]
  pub input: Vec<PathBuf>,
  /// Check if files are formatted without modifying them.
  #[arg(long)]
  pub check: bool,
  /// Override maximum line width.
  #[arg(long, value_name = "WIDTH")]
  pub line_width: Option<u32>,
  /// Override indent width in spaces.
  #[arg(long, value_name = "WIDTH")]
  pub indent_width: Option<u8>,
  /// Override newline style (lf or crlf).
  #[arg(long, value_name = "STYLE", value_parser = NewLineKind::parse)]
  pub new_line: Option<NewLineKind>,
  /// Remove empty lines between consecutive balance directives.
  #[arg(long)]
  pub compact_balance_spacing: bool,
}

/// Run the formatter CLI with a custom argument iterator.
pub fn main_with_args<I, T>(args: I) -> Result<RunOutcome>
where
  I: IntoIterator<Item = T>,
  T: Into<OsString> + Clone,
{
  let parsed = Cli::parse_from(args);
  execute(parsed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunOutcome {
  pub any_changed: bool,
}

fn execute(args: Cli) -> Result<RunOutcome> {
  let cli_overrides = args.overrides();
  let config = load_configuration(&args.input, &cli_overrides)?;
  let files = collect_files(&args.input)?;
  let mut any_changed = false;

  for path in files {
    let content = fs::read_to_string(&path)
      .with_context(|| format!("Failed to read {}", path.display()))?;
    let path_display = to_posix_path(&path);
    let formatted = format(&content, &config)?;
    let changed = formatted != content;

    if args.check {
      if changed {
        any_changed = true;
        eprintln!("checking failed: {}", path_display);
      }
      continue;
    }

    if changed {
      eprintln!("formatting: {}", path_display);

      fs::write(&path, &formatted)
        .with_context(|| format!("Failed to write {}", path.display()))?;
      any_changed = true;
    }
  }

  Ok(RunOutcome { any_changed })
}

impl Cli {
  fn overrides(&self) -> CliPartialConfiguration {
    CliPartialConfiguration {
      line_width: self.line_width,
      indent_width: self.indent_width,
      new_line: self.new_line,
      compact_balance_spacing: self.compact_balance_spacing.then_some(true),
    }
  }
}

fn load_configuration(
  inputs: &[PathBuf],
  overrides: &CliPartialConfiguration,
) -> Result<Configuration> {
  let pyproject_partial = parse_pyproject_partial(inputs)?;
  Ok(resolve_final_configuration(pyproject_partial, overrides))
}

fn parse_pyproject_partial(
  inputs: &[PathBuf],
) -> Result<Option<PyprojectPartialConfiguration>> {
  if let Some(pyproject_path) = find_pyproject(inputs) {
    let content = fs::read_to_string(&pyproject_path)
      .with_context(|| format!("Failed to read {}", pyproject_path.display()))?;

    let parsed = parse_pyproject(&content)
      .with_context(|| format!("Failed to parse {}", pyproject_path.display()))?;

    if let Some(tool) = parsed.tool {
      return Ok(tool.beancount_formatter);
    }
  }

  Ok(None)
}

fn resolve_final_configuration(
  config_file: Option<PyprojectPartialConfiguration>,
  cli_opt: &CliPartialConfiguration,
) -> Configuration {
  let config_opt = config_file.unwrap_or_default();

  let final_partial = CorePartialConfiguration {
    line_width: cli_opt.line_width.or(config_opt.line_width),
    indent_width: cli_opt.indent_width.or(config_opt.indent_width),
    new_line: cli_opt.new_line.or(config_opt.new_line),
    compact_balance_spacing: cli_opt
      .compact_balance_spacing
      .or(config_opt.compact_balance_spacing),
  };

  final_partial.resolve()
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
  let metadata = fs::metadata(path)
    .with_context(|| format!("Failed to access {}", path.display()))?;

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
  #[serde(rename = "beancount-format")]
  beancount_formatter: Option<PyprojectPartialConfiguration>,
}

#[derive(Debug, Default, Clone, serde::Deserialize)]
struct PyprojectPartialConfiguration {
  #[serde(rename = "line-width")]
  line_width: Option<u32>,
  #[serde(rename = "indent-width")]
  indent_width: Option<u8>,
  #[serde(rename = "new-line-kind")]
  new_line: Option<beancount_formatter::configuration::NewLineKind>,
  #[serde(rename = "compact-balance-spacing")]
  compact_balance_spacing: Option<bool>,
}

#[derive(Debug, Default, Clone)]
struct CliPartialConfiguration {
  line_width: Option<u32>,
  indent_width: Option<u8>,
  new_line: Option<beancount_formatter::configuration::NewLineKind>,
  compact_balance_spacing: Option<bool>,
}

fn parse_pyproject(content: &str) -> Result<Pyproject, TomlError> {
  toml::from_str(content)
}

fn to_posix_path(path: &Path) -> String {
  path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_pyproject_tool_section() {
    let content = r#"
[tool.beancount-format]
  line-width = 88
  indent-width = 3
  new-line-kind = "crlf"
  compact-balance-spacing = true
"#;

    let parsed = parse_pyproject(content).expect("pyproject should parse");
    let cfg = parsed
      .tool
      .expect("tool table missing")
      .beancount_formatter
      .expect("beancount-format table missing");

    assert_eq!(cfg.line_width, Some(88));
    assert_eq!(cfg.indent_width, Some(3));
    assert_eq!(cfg.new_line, Some(NewLineKind::CRLF));
    assert_eq!(cfg.compact_balance_spacing, Some(true));
  }
  #[test]
  fn parses_partial_pyproject_tool_section() {
    let content = r#"
[tool.beancount-format]
  line-width = 88
  indent-width = 3
"#;

    let parsed = parse_pyproject(content).expect("pyproject should parse");
    let cfg = parsed
      .tool
      .expect("tool table missing")
      .beancount_formatter
      .expect("beancount-format table missing");

    assert_eq!(cfg.line_width, Some(88));
    assert_eq!(cfg.indent_width, Some(3));
    assert_eq!(cfg.new_line, None);
    assert_eq!(cfg.compact_balance_spacing, None);
  }

  #[test]
  fn parses_without_tool_section() {
    let content = r#"
[project]
name = "example"
"#;

    let parsed = parse_pyproject(content).expect("pyproject should parse");
    assert!(parsed.tool.is_none());
  }

  #[test]
  fn rejects_invalid_toml() {
    let content = "not = [valid";
    let err = parse_pyproject(content).expect_err("parse should fail");
    assert!(err.to_string().contains("expected"));
  }

  #[test]
  fn overrides_take_precedence_when_both_set() {
    let pyproject_partial = Some(PyprojectPartialConfiguration {
      line_width: Some(70),
      indent_width: Some(2),
      new_line: Some(NewLineKind::LF),
      compact_balance_spacing: Some(false),
    });
    let overrides = CliPartialConfiguration {
      line_width: Some(88),
      indent_width: Some(4),
      new_line: Some(NewLineKind::CRLF),
      compact_balance_spacing: Some(true),
    };

    let resolved = resolve_final_configuration(pyproject_partial, &overrides);

    assert_eq!(resolved.line_width, 88);
    assert_eq!(resolved.indent_width, 4);
    assert_eq!(resolved.new_line, NewLineKind::CRLF);
    assert!(resolved.compact_balance_spacing);
  }
}
