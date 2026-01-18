use std::{env, process};

use anyhow::Result;

fn main() -> Result<()> {
  let outcome = beancount_formatter_cli::main_with_args(env::args_os())?;

  if outcome.any_changed {
    process::exit(1);
  }

  Ok(())
}
