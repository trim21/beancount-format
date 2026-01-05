pub mod configuration;
pub mod ast;
mod parse;
mod format;

pub use format::format;
pub use parse::ParseError;

/// Parse file into typed directives.
///
/// This is primarily intended for tests and debugging.
pub fn parse_directives_with_meta<'a>(
	root: tree_sitter::Node,
	source: &'a str,
	filename: String,
) -> Result<Vec<ast::Directive<'a>>, parse::ParseError> {
	parse::parse_directives(root, source, filename)
}
