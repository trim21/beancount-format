use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use beancount_formatter::configuration::{Configuration, NewLineKind};
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
  /// Override prefix width for account names.
  #[arg(long, value_name = "WIDTH")]
  pub prefix_width: Option<usize>,
  /// Override numeric column width.
  #[arg(long, value_name = "WIDTH")]
  pub num_width: Option<usize>,
}

/// Run the formatter CLI using environment arguments.
pub fn main() -> Result<RunOutcome> {
  main_with_args(env::args_os())
}

/// Run the formatter CLI with a custom argument iterator.
pub fn main_with_args<I, T>(args: I) -> Result<RunOutcome>
where
  I: IntoIterator<Item = T>,
  T: Into<OsString> + Clone,
{
  let parsed = Cli::parse_from(normalize_args(args));
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
    let content = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let path_str = path.to_string_lossy();
    let formatted = format(Some(&path_str), &content, &config)?;
    let changed = formatted != content;

    if args.check {
      if changed {
        any_changed = true;
      }
      continue;
    }

    if changed {
      fs::write(&path, &formatted).with_context(|| format!("Failed to write {}", path.display()))?;
      any_changed = true;
    }
  }

  Ok(RunOutcome { any_changed })
}

impl Cli {
  fn overrides(&self) -> PartialConfiguration {
    PartialConfiguration {
      line_width: self.line_width,
      indent_width: self.indent_width,
      new_line_kind: self.new_line,
      prefix_width: self.prefix_width,
      num_width: self.num_width,
    }
  }
}

fn load_configuration(inputs: &[PathBuf], overrides: &PartialConfiguration) -> Result<Configuration> {
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

  overrides.apply(&mut config);

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

#[derive(Debug, Default, Clone, serde::Deserialize)]
struct PartialConfiguration {
  line_width: Option<u32>,
  indent_width: Option<u8>,
  new_line_kind: Option<beancount_formatter::configuration::NewLineKind>,
  prefix_width: Option<usize>,
  num_width: Option<usize>,
}

impl PartialConfiguration {
  fn apply(&self, config: &mut Configuration) {
    config.line_width = self.line_width.unwrap_or(config.line_width);
    config.indent_width = self.indent_width.unwrap_or(config.indent_width);
    config.new_line = self.new_line_kind.unwrap_or(config.new_line);
    config.prefix_width = self.prefix_width.or(config.prefix_width);
    config.num_width = self.num_width.or(config.num_width);
  }
}

fn parse_pyproject(content: &str) -> Result<Pyproject, TomlError> {
  toml::from_str(content)
}

fn normalize_args<I, T>(args: I) -> Vec<OsString>
where
  I: IntoIterator<Item = T>,
  T: Into<OsString>,
{
  let mut values: Vec<OsString> = args.into_iter().map(Into::into).collect();

  if values.is_empty()
    || values
      .first()
      .map(|first| first.to_string_lossy().starts_with('-'))
      .unwrap_or(true)
  {
    values.insert(0, OsString::from("beancount-formatter"));
  }

  values
}
