use serde::{Deserialize, Serialize};
use super::NewLineKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
  pub line_width: u32,
  pub use_tabs: bool,
  pub indent_width: u8,
  pub new_line_kind: NewLineKind,
}
