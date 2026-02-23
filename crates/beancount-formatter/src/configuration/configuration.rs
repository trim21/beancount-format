use super::NewLineKind;
use serde::{Deserialize, Serialize};

pub const DEFAULT_LINE_WIDTH: u32 = 70;
pub const DEFAULT_INDENT_WIDTH: u8 = 2;
pub const DEFAULT_NEW_LINE_KIND: NewLineKind = NewLineKind::LF;
pub const DEFAULT_COMPACT_BALANCE_SPACING: bool = false;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
  pub line_width: u32,
  pub indent_width: u8,
  pub new_line: NewLineKind,
  pub compact_balance_spacing: bool,
}

#[derive(Debug, Default, Clone)]
pub struct PartialConfiguration {
  pub line_width: Option<u32>,
  pub indent_width: Option<u8>,
  pub new_line: Option<NewLineKind>,
  pub compact_balance_spacing: Option<bool>,
}

impl PartialConfiguration {
  pub fn merge(self, other: Self) -> Self {
    Self {
      line_width: other.line_width.or(self.line_width),
      indent_width: other.indent_width.or(self.indent_width),
      new_line: other.new_line.or(self.new_line),
      compact_balance_spacing: other
        .compact_balance_spacing
        .or(self.compact_balance_spacing),
    }
  }

  pub fn resolve(self) -> Configuration {
    Configuration {
      line_width: self.line_width.unwrap_or(DEFAULT_LINE_WIDTH),
      indent_width: self.indent_width.unwrap_or(DEFAULT_INDENT_WIDTH),
      new_line: self.new_line.unwrap_or(DEFAULT_NEW_LINE_KIND),
      compact_balance_spacing: self
        .compact_balance_spacing
        .unwrap_or(DEFAULT_COMPACT_BALANCE_SPACING),
    }
  }
}

impl Default for Configuration {
  fn default() -> Self {
    Self {
      line_width: DEFAULT_LINE_WIDTH,
      indent_width: DEFAULT_INDENT_WIDTH,
      new_line: DEFAULT_NEW_LINE_KIND,
      compact_balance_spacing: DEFAULT_COMPACT_BALANCE_SPACING,
    }
  }
}
