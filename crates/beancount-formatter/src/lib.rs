pub mod ast;
pub mod configuration;
mod format;
mod parse;

pub use format::format;
pub use beancount_parser::ParseError;

/// Parse file into typed directives.
///
/// This is primarily intended for tests and debugging.
pub fn parse_directives_with_meta<'a>(
  root: tree_sitter::Node,
  source: &'a str,
  filename: String,
) -> Result<Vec<ast::Directive<'a>>, ParseError> {
  beancount_parser::parse_directives_with_meta(root, source, filename)
}
