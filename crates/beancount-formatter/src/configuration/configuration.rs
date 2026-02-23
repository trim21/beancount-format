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
  #[serde(rename = "new_line")]
  pub new_line: NewLineKind,
  pub compact_balance_spacing: bool,
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
