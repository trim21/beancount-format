use super::NewLineKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
  pub line_width: u32,
  pub indent_width: u8,
  pub new_line_kind: NewLineKind,
  #[serde(default)]
  pub prefix_width: Option<usize>,
  #[serde(default)]
  pub num_width: Option<usize>,
  #[serde(default)]
  pub currency_column: Option<usize>,
  #[serde(default)]
  pub account_amount_spacing: Option<usize>,
  #[serde(default)]
  pub number_currency_spacing: Option<usize>,
}
