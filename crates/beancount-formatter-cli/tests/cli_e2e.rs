use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::{ord::eq, prelude::*};

const UNFORMATTED: &str = "2010-01-01 open\tAssets:Cash   \n";
const FORMATTED: &str = "2010-01-01 open Assets:Cash\n";

fn cli_cmd() -> Result<Command> {
  #[allow(deprecated)]
  {
    Command::cargo_bin("beancount-formatter").map_err(Into::into)
  }
}

#[test]
fn exits_zero_and_echoes_when_already_formatted() -> Result<()> {
  let temp = assert_fs::TempDir::new()?;
  let file = temp.child("already.bean");
  file.write_str(FORMATTED)?;

  let mut cmd = cli_cmd()?;
  cmd.arg(file.path());

  cmd
    .assert()
    .success()
    .stdout(eq(FORMATTED))
    .stderr(predicate::str::is_empty());

  file.assert(eq(FORMATTED));
  Ok(())
}

#[test]
fn prints_formatted_output_and_nonzero_when_changes() -> Result<()> {
  let temp = assert_fs::TempDir::new()?;
  let file = temp.child("needs-format.beancount");
  file.write_str(UNFORMATTED)?;

  let mut cmd = cli_cmd()?;
  cmd.arg(file.path());

  cmd
    .assert()
    .failure()
    .stdout(eq(FORMATTED))
    .stderr(predicate::str::is_empty());

  file.assert(eq(UNFORMATTED));
  Ok(())
}

#[test]
fn rewrites_file_in_place_when_requested() -> Result<()> {
  let temp = assert_fs::TempDir::new()?;
  let file = temp.child("rewrite.beancount");
  file.write_str(UNFORMATTED)?;

  let mut cmd = cli_cmd()?;
  cmd.arg("--in-place").arg(file.path());

  cmd
    .assert()
    .failure()
    .stdout(predicate::str::is_empty())
    .stderr(predicate::str::is_empty());

  file.assert(eq(FORMATTED));
  Ok(())
}

#[test]
fn respects_pyproject_configuration() -> Result<()> {
  let temp = assert_fs::TempDir::new()?;
  temp.child("pyproject.toml").write_str(
    r#"
[tool.beancount-formatter]
new_line_kind = "crlf"
"#,
  )?;

  let file = temp.child("configurable.beancount");
  file.write_str("2010-01-01 open Assets:Cash\n")?;

  let mut cmd = cli_cmd()?;
  cmd.current_dir(temp.path());
  cmd.arg(file.path());

  cmd
    .assert()
    .failure()
    .stdout(eq("2010-01-01 open Assets:Cash\r\n"))
    .stderr(predicate::str::is_empty());

  file.assert(eq("2010-01-01 open Assets:Cash\n"));
  Ok(())
}
