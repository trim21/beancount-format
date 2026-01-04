use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use beancount_formatter::configuration::Configuration;
use beancount_formatter::format;
use clap::Parser;

/// Simple CLI to format beancount files.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
  /// Path to the beancount file to format.
  input: PathBuf,
  /// Write changes back to the file instead of printing to stdout.
  #[arg(short, long)]
  in_place: bool,
}

fn main() -> Result<()> {
  let args = Cli::parse();
  let content = fs::read_to_string(&args.input)?;
  let config = Configuration::default();
  let path = args.input.to_string_lossy();
  let formatted = format(Some(&path), &content, &config)?;

  if formatted == content {
    if !args.in_place {
      print!("{}", content);
    }
  } else if args.in_place {
    fs::write(&args.input, formatted)?;
  } else {
    print!("{}", formatted);
  }

  Ok(())
}
