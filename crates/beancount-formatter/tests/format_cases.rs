#[test]
fn format_and_check_fixtures() {
  use std::ffi::OsStr;
  use std::fs;
  use std::path::Path;

  use beancount_formatter::configuration::{Configuration, NewLineKind};
  use beancount_formatter::format;
  use serde::Deserialize;

  #[derive(Debug, Default, Deserialize)]
  #[serde(default)]
  struct PartialConfiguration {
    line_width: Option<u32>,
    indent_width: Option<u8>,
    new_line_kind: Option<NewLineKind>,
    prefix_width: Option<usize>,
    num_width: Option<usize>,
    currency_column: Option<usize>,
    account_amount_spacing: Option<usize>,
    number_currency_spacing: Option<usize>,
  }

  impl PartialConfiguration {
    fn apply_to(self, mut config: Configuration) -> Configuration {
      if let Some(v) = self.line_width {
        config.line_width = v;
      }
      if let Some(v) = self.indent_width {
        config.indent_width = v;
      }
      if let Some(v) = self.new_line_kind {
        config.new_line_kind = v;
      }
      if let Some(v) = self.prefix_width {
        config.prefix_width = Some(v);
      }
      if let Some(v) = self.num_width {
        config.num_width = Some(v);
      }
      if let Some(v) = self.currency_column {
        config.currency_column = Some(v);
      }
      if let Some(v) = self.account_amount_spacing {
        config.account_amount_spacing = Some(v);
      }
      if let Some(v) = self.number_currency_spacing {
        config.number_currency_spacing = Some(v);
      }

      config
    }
  }

  fn assert_eq_with_diff(expected: &str, actual: &str) {
    if expected == actual {
      return;
    }

    eprintln!("=== expected ===\n{}", expected);
    eprintln!("=== actual ===\n{}", actual);
    eprintln!("=== line diff ===");

    let expected_lines: Vec<&str> = expected.split_terminator('\n').collect();
    let actual_lines: Vec<&str> = actual.split_terminator('\n').collect();
    let max = expected_lines.len().max(actual_lines.len());

    for i in 0..max {
      let expected_line = expected_lines.get(i).copied().unwrap_or("");
      let actual_line = actual_lines.get(i).copied().unwrap_or("");
      if expected_line == actual_line {
        continue;
      }
      eprintln!("line {}:", i + 1);
      eprintln!("- expected: {:?}", expected_line);
      eprintln!("+ actual  : {:?}", actual_line);
    }

    panic!("text mismatch; see diff above");
  }

  fn run_case(input_path: &Path) {
    let file_name = input_path
      .file_name()
      .and_then(|s| s.to_str())
      .expect("input path has non-utf8 filename");

    let case_name = file_name
      .strip_suffix(".input.bean")
      .expect("input file must end with .input.bean");

    let dir = input_path.parent().expect("input file has no parent dir");

    let config_path = dir.join(format!("{case_name}.config.json"));
    let expected_path = dir.join(format!("{case_name}.expected.bean"));

    let input = fs::read_to_string(input_path)
      .unwrap_or_else(|e| panic!("Failed to read input {}: {e}", input_path.display()));

    let config = if config_path.exists() {
      let json = fs::read_to_string(&config_path)
        .unwrap_or_else(|e| panic!("Failed to read config {}: {e}", config_path.display()));
      let partial: PartialConfiguration = serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("Invalid JSON in {}: {e}", config_path.display()));
      partial.apply_to(Configuration::default())
    } else {
      Configuration::default()
    };

    // Use case name as the filename for nicer error messages and meta handling.
    let formatted = format(Some(&format!("{case_name}.bean")), &input, &config)
      .unwrap_or_else(|e| panic!("format() failed for {case_name}: {e:?}"));

    if !expected_path.exists() {
      fs::write(&expected_path, &formatted)
        .unwrap_or_else(|e| panic!("Failed to write expected {}: {e}", expected_path.display()));

      panic!(
        "Missing expected file; wrote formatted output to {}",
        expected_path.display()
      );
    }

    let expected = fs::read_to_string(&expected_path)
      .unwrap_or_else(|e| panic!("Failed to read expected {}: {e}", expected_path.display()));

    assert_eq_with_diff(&expected, &formatted);
  }

  let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/format-and-check");
  let mut input_files = vec![];
  for entry in fs::read_dir(&fixtures_dir)
    .unwrap_or_else(|e| panic!("Failed to read fixtures dir {}: {e}", fixtures_dir.display()))
  {
    let entry = entry.unwrap_or_else(|e| panic!("Failed to read fixtures dir entry: {e}"));
    let path = entry.path();
    if path.extension() != Some(OsStr::new("bean")) {
      continue;
    }
    if !path
      .file_name()
      .and_then(|s| s.to_str())
      .is_some_and(|s| s.ends_with(".input.bean"))
    {
      continue;
    }
    input_files.push(path);
  }

  input_files.sort();
  assert!(
    !input_files.is_empty(),
    "No fixtures found in {} (expected at least one `*.input.bean`)",
    fixtures_dir.display()
  );

  for input_path in input_files {
    run_case(&input_path);
  }
}

