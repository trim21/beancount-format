use std::fmt::{self, Display};
use std::str::FromStr;

use dprint_core::configuration::NewLineKind as CoreNewLineKind;
use serde::{Deserialize, Serialize};

/// Newline style accepted by the beancount formatter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewLineKind {
  /// Line feed ("\n").
  #[serde(rename = "lf")]
  LF,
  /// Carriage return + line feed ("\r\n").
  #[serde(rename = "crlf")]
  CRLF,
}

impl NewLineKind {
  pub fn as_str(&self) -> &'static str {
    match self {
      NewLineKind::LF => "lf",
      NewLineKind::CRLF => "crlf",
    }
  }

  /// Parse a newline kind from a string. Accepts case-insensitive "lf" or "crlf".
  pub fn parse(text: &str) -> Result<Self, String> {
    match text.to_ascii_lowercase().as_str().trim() {
      "lf" => Ok(NewLineKind::LF),
      "crlf" => Ok(NewLineKind::CRLF),
      other => Err(format!("Unsupported new_line_kind: {}", other)),
    }
  }
}

impl Display for NewLineKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

impl FromStr for NewLineKind {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::parse(s)
  }
}

impl From<NewLineKind> for CoreNewLineKind {
  fn from(value: NewLineKind) -> Self {
    match value {
      NewLineKind::LF => CoreNewLineKind::LineFeed,
      NewLineKind::CRLF => CoreNewLineKind::CarriageReturnLineFeed,
    }
  }
}

impl From<CoreNewLineKind> for NewLineKind {
  fn from(value: CoreNewLineKind) -> Self {
    match value {
      CoreNewLineKind::LineFeed => NewLineKind::LF,
      CoreNewLineKind::CarriageReturnLineFeed => NewLineKind::CRLF,
      // The formatter only exposes lf and crlf; map other variants to defaults if added upstream.
      _ => NewLineKind::LF,
    }
  }
}
