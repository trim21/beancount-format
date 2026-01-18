use anyhow::Result;
use assert_cmd::{Command, cargo::cargo_bin_cmd};
use assert_fs::prelude::*;
use predicates::{ord::eq, prelude::*};

const UNFORMATTED: &str = "2010-01-01 open\tAssets:Cash   \n";
const FORMATTED: &str = "2010-01-01 open Assets:Cash\n";

#[test]
fn exits_zero_when_already_formatted() -> Result<()> {
  let temp = assert_fs::TempDir::new()?;
  let file = temp.child("already.bean");
  file.write_str(FORMATTED)?;

  let mut cmd: Command = cargo_bin_cmd!("beancount-format");
  cmd.arg(file.path());

  cmd
    .assert()
    .success()
    .stdout(predicate::str::is_empty())
    .stderr(predicate::str::is_empty());

  file.assert(eq(FORMATTED));
  Ok(())
}

#[test]
fn rewrites_and_nonzero_when_changes() -> Result<()> {
  let temp = assert_fs::TempDir::new()?;
  let file = temp.child("needs-format.beancount");
  file.write_str(UNFORMATTED)?;

  let mut cmd: Command = cargo_bin_cmd!("beancount-format");
  cmd.arg(file.path());

  cmd
    .assert()
    .failure()
    .stdout(predicate::str::is_empty())
    .stderr(predicate::str::is_empty());

  file.assert(eq(FORMATTED));
  Ok(())
}

#[test]
fn check_mode_reports_without_writing() -> Result<()> {
  let temp = assert_fs::TempDir::new()?;
  let file = temp.child("rewrite.beancount");
  file.write_str(UNFORMATTED)?;

  let mut cmd: Command = cargo_bin_cmd!("beancount-format");
  cmd.arg("--check").arg(file.path());

  cmd
    .assert()
    .failure()
    .stdout(predicate::str::is_empty())
    .stderr(predicate::str::is_empty());

  file.assert(eq(UNFORMATTED));
  Ok(())
}

#[test]
fn respects_pyproject_configuration() -> Result<()> {
  let temp = assert_fs::TempDir::new()?;
  temp.child("pyproject.toml").write_str(
    r#"
[tool.beancount-format]
new-line-kind = "crlf"
"#,
  )?;

  let file = temp.child("configurable.beancount");
  file.write_str("2010-01-01 open Assets:Cash\n")?;

  let mut cmd: Command = cargo_bin_cmd!("beancount-format");
  cmd.current_dir(temp.path());
  cmd.arg(file.path());

  cmd
    .assert()
    .failure()
    .stdout(predicate::str::is_empty())
    .stderr(predicate::str::is_empty());

  file.assert(eq("2010-01-01 open Assets:Cash\r\n"));
  Ok(())
}
