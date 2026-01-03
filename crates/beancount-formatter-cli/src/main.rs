use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use beancount_formatter::configuration::ConfigurationBuilder;
use beancount_formatter::format_text;
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
  let config = ConfigurationBuilder::new().build();
  let maybe_formatted = format_text(&args.input, &content, &config)?;

  match maybe_formatted {
    Some(formatted) => {
      if args.in_place {
        fs::write(&args.input, formatted)?;
      } else {
        print!("{}", formatted);
      }
    }
    None => {
      if !args.in_place {
        print!("{}", content);
      }
    }
  }

  Ok(())
}
