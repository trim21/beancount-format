use beancount_formatter::configuration::Configuration;
use beancount_formatter::configuration::NewLineKind;
use beancount_formatter::format;

#[test]
fn formats_without_changes_returns_none() {
  let config = Configuration::default();

  let result = format(Some("example.beancount"), "2010-01-01 open Assets:Cash\n", &config).unwrap();

  assert_eq!(result, "2010-01-01 open Assets:Cash\n");
}

#[test]
fn normalizes_tabs_and_trailing_spaces() {
  let config = Configuration::default();

  let input = "2010-01-01 open\tAssets:Cash   \n";
  let expected = "2010-01-01 open Assets:Cash\n";

  let result = format(Some("example.beancount"), input, &config).unwrap();
  assert_eq_with_diff(expected, result.as_str());
}

#[test]
fn support_input_without_eol() {
  let config = Configuration::default();

  let input = "2010-01-01 open Assets:Cash CNY";
  let expected = "2010-01-01 open Assets:Cash CNY\n";

  let result = format(Some("example.beancount"), input, &config).unwrap();
  assert_eq_with_diff(expected, result.as_str());
}

#[test]
fn applies_configured_crlf_newlines() {
  let config = Configuration {
    new_line_kind: NewLineKind::CRLF,
    ..Configuration::default()
  };

  let input = "2010-01-01 open Assets:Cash\n";
  let expected = "2010-01-01 open Assets:Cash\r\n";

  let result = format(Some("example.beancount"), input, &config).unwrap();
  assert_eq_with_diff(expected, result.as_str());
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
