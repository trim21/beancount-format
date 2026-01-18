use std::process;

use anyhow::Result;

fn main() -> Result<()> {
  let outcome = beancount_formatter_cli::main()?;

  if outcome.any_changed {
    process::exit(1);
  }

  Ok(())
}
