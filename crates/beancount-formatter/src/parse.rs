use beancount_parser::{self as parser, ParseError};

pub fn parse_source<'a>(source: &'a str) -> Vec<parser::ast::Directive<'a>> {
  parser::parse_str(source)
}
