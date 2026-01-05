use std::borrow::Cow;

/// Byte offsets in the original source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
  pub start: usize,
  pub end: usize,
}

impl Span {
  pub fn from_range(start: usize, end: usize) -> Self {
    Self { start, end }
  }
}

/// Source location info attached to each top-level directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meta {
  pub filename: String,
  /// 1-based line number.
  pub line: usize,
  /// 1-based column number.
  pub column: usize,
}

/// A typed representation of top-level Beancount directives/entries.
///
/// This is intentionally lossy at first: we keep `span`/`raw` so we can fall back
/// while we incrementally implement full formatting rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive<'a> {
  Open(Open<'a>),
  Close(Close<'a>),
  Balance(Balance<'a>),
  Pad(Pad<'a>),
  Transaction(Transaction<'a>),

  Commodity(Commodity<'a>),
  Price(Price<'a>),
  Event(Event<'a>),
  Query(Query<'a>),
  Note(Note<'a>),
  Document(Document<'a>),
  Custom(Custom<'a>),

  Option(OptionDirective<'a>),
  Include(Include<'a>),
  Plugin(Plugin<'a>),
  Pushtag(TagDirective<'a>),
  Poptag(TagDirective<'a>),
  Pushmeta(Pushmeta<'a>),
  Popmeta(Popmeta<'a>),

  /// Any entry/directive we don't parse yet.
  Raw(Raw<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raw<'a> {
  pub meta: Meta,
  pub kind: Cow<'a, str>,
  pub span: Span,
  pub text: Cow<'a, str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Open<'a> {
  pub meta: Meta,
  pub span: Span,
  pub date: Cow<'a, str>,
  pub account: Cow<'a, str>,
  pub currencies: Vec<Cow<'a, str>>,
  pub opt_booking: Option<Cow<'a, str>>,
  pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Close<'a> {
  pub meta: Meta,
  pub span: Span,
  pub date: Cow<'a, str>,
  pub account: Cow<'a, str>,
  pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Balance<'a> {
  pub meta: Meta,
  pub span: Span,
  pub date: Cow<'a, str>,
  pub account: Cow<'a, str>,
  pub amount: Cow<'a, str>,
  pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pad<'a> {
  pub meta: Meta,
  pub span: Span,
  pub date: Cow<'a, str>,
  pub account: Cow<'a, str>,
  pub from_account: Cow<'a, str>,
  pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction<'a> {
  pub meta: Meta,
  pub span: Span,
  pub date: Cow<'a, str>,
  /// Transaction flag/token (e.g. `*`, `!`) when present.
  pub txn: Option<Cow<'a, str>>,
  pub payee: Option<Cow<'a, str>>,
  pub narration: Cow<'a, str>,
  pub tags_links: Option<Cow<'a, str>>,
  pub comment: Option<Cow<'a, str>>,
  /// All tag/link groups attached to the transaction (inline and indented lines).
  pub tags_links_lines: Vec<Cow<'a, str>>,
  /// All comments attached to the transaction (inline and indented lines).
  pub comments: Vec<Cow<'a, str>>,
  /// Metadata key/value lines attached to the transaction.
  pub key_values: Vec<KeyValue<'a>>,
  /// Postings within the transaction, in source order.
  pub postings: Vec<Posting<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValue<'a> {
  pub meta: Meta,
  pub span: Span,
  pub key: Cow<'a, str>,
  pub value: Cow<'a, str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting<'a> {
  pub meta: Meta,
  pub span: Span,
  pub opt_flag: Option<Cow<'a, str>>,
  pub account: Cow<'a, str>,
  pub amount: Option<Cow<'a, str>>,
  pub cost_spec: Option<Cow<'a, str>>,
  pub price_operator: Option<Cow<'a, str>>,
  pub price_annotation: Option<Cow<'a, str>>,
  pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commodity<'a> {
  pub meta: Meta,
  pub span: Span,
  pub date: Cow<'a, str>,
  pub currency: Cow<'a, str>,
  pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Price<'a> {
  pub meta: Meta,
  pub span: Span,
  pub date: Cow<'a, str>,
  pub currency: Cow<'a, str>,
  pub amount: Cow<'a, str>,
  pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event<'a> {
  pub meta: Meta,
  pub span: Span,
  pub date: Cow<'a, str>,
  pub event_type: Cow<'a, str>,
  pub desc: Cow<'a, str>,
  pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query<'a> {
  pub meta: Meta,
  pub span: Span,
  pub date: Cow<'a, str>,
  pub name: Cow<'a, str>,
  pub query: Cow<'a, str>,
  pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note<'a> {
  pub meta: Meta,
  pub span: Span,
  pub date: Cow<'a, str>,
  pub account: Cow<'a, str>,
  pub note: Cow<'a, str>,
  pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document<'a> {
  pub meta: Meta,
  pub span: Span,
  pub date: Cow<'a, str>,
  pub account: Cow<'a, str>,
  pub filename: Cow<'a, str>,
  pub tags_links: Option<Cow<'a, str>>,
  pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Custom<'a> {
  pub meta: Meta,
  pub span: Span,
  pub date: Cow<'a, str>,
  pub name: Cow<'a, str>,
  pub values: Vec<Cow<'a, str>>,
  pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionDirective<'a> {
  pub meta: Meta,
  pub span: Span,
  pub key: Cow<'a, str>,
  pub value: Cow<'a, str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Include<'a> {
  pub meta: Meta,
  pub span: Span,
  pub filename: Cow<'a, str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plugin<'a> {
  pub meta: Meta,
  pub span: Span,
  pub name: Cow<'a, str>,
  pub config: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagDirective<'a> {
  pub meta: Meta,
  pub span: Span,
  pub tag: Cow<'a, str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pushmeta<'a> {
  pub meta: Meta,
  pub span: Span,
  pub key_value: Cow<'a, str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Popmeta<'a> {
  pub meta: Meta,
  pub span: Span,
  pub key: Cow<'a, str>,
}
