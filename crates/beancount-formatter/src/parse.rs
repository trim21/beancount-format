use tree_sitter::Node;

use crate::ast::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
  pub filename: String,
  pub line: usize,
  pub column: usize,
  pub message: String,
}

impl std::fmt::Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}:{}: {}", self.filename, self.line, self.column, self.message)
  }
}

impl std::error::Error for ParseError {}

type Result<T> = std::result::Result<T, ParseError>;

fn meta(node: Node, filename: &str) -> Meta {
  let p = node.start_position();
  Meta {
    filename: filename.to_owned(),
    line: p.row + 1,
    column: p.column + 1,
  }
}

fn parse_error(node: Node, filename: &str, message: impl Into<String>) -> ParseError {
  let p = node.start_position();
  ParseError {
    filename: filename.to_owned(),
    line: p.row + 1,
    column: p.column + 1,
    message: message.into(),
  }
}

fn slice<'a>(node: Node, source: &'a str) -> &'a str {
  &source[node.start_byte()..node.end_byte()]
}

fn span(node: Node) -> Span {
  Span::from_range(node.start_byte(), node.end_byte())
}

fn field_text<'a>(node: Node, field: &str, source: &'a str) -> Option<std::borrow::Cow<'a, str>> {
  node
    .child_by_field_name(field)
    .map(|n| std::borrow::Cow::Borrowed(slice(n, source)))
}

fn required_field_text<'a>(
  node: Node,
  field: &str,
  source: &'a str,
  filename: &str,
) -> Result<std::borrow::Cow<'a, str>> {
  field_text(node, field, source).ok_or_else(|| {
    parse_error(
      node,
      filename,
      format!("missing field `{}` in `{}`", field, node.kind()),
    )
  })
}

fn first_named_child_text<'a>(node: Node, source: &'a str) -> Option<std::borrow::Cow<'a, str>> {
  let mut cursor = node.walk();
  node
    .named_children(&mut cursor)
    .next()
    .map(|n| std::borrow::Cow::Borrowed(slice(n, source)))
}

pub fn parse_directives<'a>(root: Node, source: &'a str, filename: String) -> Result<Vec<Directive<'a>>> {
  // The grammar's root rule name is `file`. If callers pass a different node
  // (e.g. a single directive node), return a structured error.
  if root.kind() != "file" {
    let p = root.start_position();
    return Err(ParseError {
      filename,
      line: p.row + 1,
      column: p.column + 1,
      message: format!("expected root node kind `file`, got `{}`", root.kind()),
    });
  }

  let mut cursor = root.walk();
  root
    .named_children(&mut cursor)
    .map(|node| parse_top_level(node, source, &filename))
    .collect::<Result<Vec<_>>>()
}

fn parse_top_level<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  match node.kind() {
    // entries
    "open" => parse_open(node, source, filename),
    "close" => parse_close(node, source, filename),
    "balance" => parse_balance(node, source, filename),
    "pad" => parse_pad(node, source, filename),
    "transaction" => parse_transaction(node, source, filename),
    "document" => parse_document(node, source, filename),
    "note" => parse_note(node, source, filename),
    "event" => parse_event(node, source, filename),
    "price" => parse_price(node, source, filename),
    "commodity" => parse_commodity(node, source, filename),
    "query" => parse_query(node, source, filename),
    "custom" => parse_custom(node, source, filename),

    // directives
    "option" => parse_option(node, source, filename),
    "include" => parse_include(node, source, filename),
    "plugin" => parse_plugin(node, source, filename),
    "pushtag" => parse_pushtag(node, source, filename),
    "poptag" => parse_poptag(node, source, filename),
    "pushmeta" => parse_pushmeta(node, source, filename),
    "popmeta" => parse_popmeta(node, source, filename),

    // Known non-directive top-level nodes.
    "section" | "comment" => Ok(raw(node, source, filename)),

    other => Err(parse_error(
      node,
      filename,
      format!("unknown directive node kind `{}`", other),
    )),
  }
}

fn raw<'a>(node: Node, source: &'a str, filename: &str) -> Directive<'a> {
  Directive::Raw(Raw {
    meta: meta(node, filename),
    kind: std::borrow::Cow::Borrowed(node.kind()),
    span: span(node),
    text: std::borrow::Cow::Borrowed(slice(node, source)),
  })
}

fn parse_open<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  let date = required_field_text(node, "date", source, filename)?;
  let account = required_field_text(node, "account", source, filename)?;

  let opt_booking = field_text(node, "opt_booking", source);
  let comment = field_text(node, "comment", source);

  // NOTE: `currency` is a token, so it is not a named node in this grammar.
  // To avoid losing it, we parse currencies from the raw text of the field.
  let currencies = node
    .child_by_field_name("currencies")
    .map(|curr_node| parse_currencies_from_text(slice(curr_node, source)))
    .unwrap_or_default();

  Ok(Directive::Open(Open {
    meta: meta(node, filename),
    span: span(node),
    date,
    account,
    currencies,
    opt_booking,
    comment,
  }))
}

fn parse_currencies_from_text(text: &str) -> Vec<std::borrow::Cow<'_, str>> {
  // Currency token regex in grammar:
  // [A-Z]([A-Z0-9\'\._\-]{0,22}[A-Z0-9])?
  // Here we do a best-effort scan that matches the common case.
  let mut out = Vec::new();
  let bytes = text.as_bytes();
  let mut i = 0;

  while i < bytes.len() {
    let b = bytes[i];
    if !b.is_ascii_uppercase() {
      i += 1;
      continue;
    }

    let start = i;
    i += 1;
    while i < bytes.len() {
      let b = bytes[i];
      if b.is_ascii_uppercase() || b.is_ascii_digit() || matches!(b, b'\'' | b'.' | b'_' | b'-') {
        i += 1;
      } else {
        break;
      }
    }

    // Keep only if ends with [A-Z0-9] per grammar.
    if i > start {
      let last = bytes[i - 1];
      if (last.is_ascii_uppercase() || last.is_ascii_digit())
        && let Ok(s) = std::str::from_utf8(&bytes[start..i])
      {
        out.push(std::borrow::Cow::Owned(s.to_string()));
      }
    }
  }

  out
}

fn parse_close<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  Ok(Directive::Close(Close {
    meta: meta(node, filename),
    span: span(node),
    date: required_field_text(node, "date", source, filename)?,
    account: required_field_text(node, "account", source, filename)?,
    comment: field_text(node, "comment", source),
  }))
}

fn parse_balance<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  Ok(Directive::Balance(Balance {
    meta: meta(node, filename),
    span: span(node),
    date: required_field_text(node, "date", source, filename)?,
    account: required_field_text(node, "account", source, filename)?,
    amount: field_text(node, "amount", source)
      .or_else(|| first_named_child_text(node, source))
      .ok_or_else(|| parse_error(node, filename, "missing amount"))?,
    comment: field_text(node, "comment", source),
  }))
}

fn parse_pad<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  Ok(Directive::Pad(Pad {
    meta: meta(node, filename),
    span: span(node),
    date: required_field_text(node, "date", source, filename)?,
    account: required_field_text(node, "account", source, filename)?,
    from_account: required_field_text(node, "from_account", source, filename)?,
    comment: field_text(node, "comment", source),
  }))
}

fn parse_commodity<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  Ok(Directive::Commodity(Commodity {
    meta: meta(node, filename),
    span: span(node),
    date: required_field_text(node, "date", source, filename)?,
    currency: required_field_text(node, "currency", source, filename)?,
    comment: field_text(node, "comment", source),
  }))
}

fn parse_price<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  Ok(Directive::Price(Price {
    meta: meta(node, filename),
    span: span(node),
    date: required_field_text(node, "date", source, filename)?,
    currency: required_field_text(node, "currency", source, filename)?,
    amount: required_field_text(node, "amount", source, filename)?,
    comment: field_text(node, "comment", source),
  }))
}

fn parse_event<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  Ok(Directive::Event(Event {
    meta: meta(node, filename),
    span: span(node),
    date: required_field_text(node, "date", source, filename)?,
    event_type: required_field_text(node, "type", source, filename)?,
    desc: required_field_text(node, "desc", source, filename)?,
    comment: field_text(node, "comment", source),
  }))
}

fn parse_query<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  Ok(Directive::Query(Query {
    meta: meta(node, filename),
    span: span(node),
    date: required_field_text(node, "date", source, filename)?,
    name: required_field_text(node, "name", source, filename)?,
    query: required_field_text(node, "query", source, filename)?,
    comment: field_text(node, "comment", source),
  }))
}

fn parse_note<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  Ok(Directive::Note(Note {
    meta: meta(node, filename),
    span: span(node),
    date: required_field_text(node, "date", source, filename)?,
    account: required_field_text(node, "account", source, filename)?,
    note: required_field_text(node, "note", source, filename)?,
    comment: field_text(node, "comment", source),
  }))
}

fn parse_document<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  Ok(Directive::Document(Document {
    meta: meta(node, filename),
    span: span(node),
    date: required_field_text(node, "date", source, filename)?,
    account: required_field_text(node, "account", source, filename)?,
    filename: required_field_text(node, "filename", source, filename)?,
    tags_links: field_text(node, "tags_links", source),
    comment: field_text(node, "comment", source),
  }))
}

fn parse_custom<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  // `custom_value_list` is modeled as repeat1(custom_value) in the grammar.
  // Collect all `custom_value` named children.
  let mut cursor = node.walk();
  let values = node
    .named_children(&mut cursor)
    .filter(|n| n.kind() == "custom_value")
    .map(|n| std::borrow::Cow::Borrowed(slice(n, source)))
    .collect::<Vec<_>>();

  Ok(Directive::Custom(Custom {
    meta: meta(node, filename),
    span: span(node),
    date: required_field_text(node, "date", source, filename)?,
    name: required_field_text(node, "name", source, filename)?,
    values,
    comment: field_text(node, "comment", source),
  }))
}

fn parse_option<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  Ok(Directive::Option(OptionDirective {
    meta: meta(node, filename),
    span: span(node),
    key: required_field_text(node, "key", source, filename)?,
    value: required_field_text(node, "value", source, filename)?,
  }))
}

fn parse_include<'a>(node: Node, source: &'a str, meta_filename: &str) -> Result<Directive<'a>> {
  // include: seq("include", $.string, $._eol)
  // It's not a field, so take the 1st named child (string).
  let mut cursor = node.walk();
  let filename = node
    .named_children(&mut cursor)
    .find(|n| n.kind() == "string")
    .map(|n| std::borrow::Cow::Borrowed(slice(n, source)))
    .ok_or_else(|| parse_error(node, meta_filename, "missing string"))?;

  Ok(Directive::Include(Include {
    meta: meta(node, meta_filename),
    span: span(node),
    filename,
  }))
}

fn parse_plugin<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  // plugin: seq("plugin", $.string, $._eol) | seq("plugin", $.string, $.string, $._eol)
  let mut cursor = node.walk();
  let mut strings = node
    .named_children(&mut cursor)
    .filter(|n| n.kind() == "string")
    .map(|n| std::borrow::Cow::Borrowed(slice(n, source)));

  let name = strings
    .next()
    .ok_or_else(|| parse_error(node, filename, "missing plugin name"))?;
  let config = strings.next();

  Ok(Directive::Plugin(Plugin {
    meta: meta(node, filename),
    span: span(node),
    name,
    config,
  }))
}

fn parse_pushtag<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  let mut cursor = node.walk();
  let tag = node
    .named_children(&mut cursor)
    .find(|n| n.kind() == "tag")
    .map(|n| std::borrow::Cow::Borrowed(slice(n, source)))
    .ok_or_else(|| parse_error(node, filename, "missing tag"))?;

  Ok(Directive::Pushtag(TagDirective {
    meta: meta(node, filename),
    span: span(node),
    tag,
  }))
}

fn parse_poptag<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  let mut cursor = node.walk();
  let tag = node
    .named_children(&mut cursor)
    .find(|n| n.kind() == "tag")
    .map(|n| std::borrow::Cow::Borrowed(slice(n, source)))
    .ok_or_else(|| parse_error(node, filename, "missing tag"))?;

  Ok(Directive::Poptag(TagDirective {
    meta: meta(node, filename),
    span: span(node),
    tag,
  }))
}

fn parse_pushmeta<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  Ok(Directive::Pushmeta(Pushmeta {
    meta: meta(node, filename),
    span: span(node),
    key_value: first_named_child_text(node, source).ok_or_else(|| parse_error(node, filename, "missing key_value"))?,
  }))
}

fn parse_popmeta<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  // popmeta: seq("popmeta", $.key, ":", $._eol)
  let mut cursor = node.walk();
  let key = node
    .named_children(&mut cursor)
    .find(|n| n.kind() == "key")
    .map(|n| std::borrow::Cow::Borrowed(slice(n, source)))
    .ok_or_else(|| parse_error(node, filename, "missing key"))?;

  Ok(Directive::Popmeta(Popmeta {
    meta: meta(node, filename),
    span: span(node),
    key,
  }))
}

fn parse_transaction<'a>(node: Node, source: &'a str, filename: &str) -> Result<Directive<'a>> {
  // We keep this intentionally shallow for now: ensure the node has a date and narration.
  // Different grammar versions may or may not expose field names; we support both fields and heuristics.

  let date = field_text(node, "date", source)
    .or_else(|| {
      let mut cursor = node.walk();
      node
        .named_children(&mut cursor)
        .find(|n| n.kind() == "date")
        .map(|n| std::borrow::Cow::Borrowed(slice(n, source)))
    })
    .ok_or_else(|| parse_error(node, filename, "missing date"))?;

  let txn = field_text(node, "txn", source)
    .or_else(|| field_text(node, "flag", source))
    .or_else(|| {
      let mut cursor = node.walk();
      node
        .named_children(&mut cursor)
        .find(|n| n.kind() == "txn")
        .map(|n| std::borrow::Cow::Borrowed(slice(n, source)))
    });

  let payee = field_text(node, "payee", source);
  let narration = field_text(node, "narration", source);

  let (payee, narration) = match (payee, narration) {
    (p, Some(n)) => (p, n),
    // Heuristic: take string children. If there are 2, assume payee+narration; if 1, narration only.
    (None, None) => {
      let mut cursor = node.walk();
      let mut strings = node
        .named_children(&mut cursor)
        .filter(|n| n.kind() == "string")
        .map(|n| std::borrow::Cow::Borrowed(slice(n, source)));
      let first = strings.next();
      let second = strings.next();
      match (first, second) {
        (Some(n), None) => (None, n),
        (Some(p), Some(n)) => (Some(p), n),
        _ => return Err(parse_error(node, filename, "missing narration")),
      }
    }
    (p, None) => {
      // payee without narration isn't valid; treat as parse error.
      let _ = p;
      return Err(parse_error(node, filename, "missing narration"));
    }
  };

  Ok(Directive::Transaction(Transaction {
    meta: meta(node, filename),
    span: span(node),
    date,
    txn,
    payee,
    narration,
    tags_links: field_text(node, "tags_links", source),
    comment: field_text(node, "comment", source),
  }))
}
