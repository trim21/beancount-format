pub mod configuration;
mod format;
mod parse;

pub use beancount_parser::ParseError;
pub use format::format;

/// Parse file into typed directives.
///
/// This is primarily intended for tests and debugging.
pub fn parse_directives_with_meta<'a>(
  root: tree_sitter::Node,
  source: &'a str,
  filename: String,
) -> Result<Vec<beancount_parser::ast::Directive<'a>>, ParseError> {
  beancount_parser::parse_directives_with_meta(root, source, filename)
}
