use beancount_parser::{self as parser, ParseError};

pub type Result<T> = std::result::Result<T, ParseError>;

pub fn parse_source<'a>(source: &'a str, filename: &str) -> Result<Vec<parser::ast::Directive<'a>>> {
  parser::parse_str(source, filename)
}
