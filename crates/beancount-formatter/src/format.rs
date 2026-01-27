use anyhow::Result;

use crate::configuration::{Configuration, NewLineKind};
use crate::parse::parse_source;
use beancount_parser::ast::{self, Directive, PriceOperator, WithSpan};

/// Simple string writer to avoid building large intermediate vectors before concatenation.
struct Writer {
  buf: String,
}

fn format_open(writer: &mut Writer, d: &ast::Open<'_>, config: &Configuration) {
  let comment_col = config.line_width as usize;
  let mut line = join_parts([
    Some(to_part(&d.date)),
    Some("open".to_string()),
    Some(to_part(&d.account)),
  ]);
  line = align_trailing(line, format_currencies(&d.currencies), comment_col);
  if let Some(comment) = &d.comment {
    line = append_comment(line, &format_comment(comment), config, true);
  }
  writer.write_str(&line);
}

fn format_close(writer: &mut Writer, d: &ast::Close<'_>, config: &Configuration) {
  let mut line = join_parts([
    Some(to_part(&d.date)),
    Some("close".to_string()),
    Some(to_part(&d.account)),
  ]);
  if let Some(comment) = &d.comment {
    line = append_comment(line, &format_comment(comment), config, false);
  }
  writer.write_str(&line);
}

fn format_balance(writer: &mut Writer, d: &ast::Balance<'_>, config: &Configuration) {
  let comment_col = config.line_width as usize;
  let mut line = join_parts([
    Some(to_part(&d.date)),
    Some("balance".to_string()),
    Some(to_part(&d.account)),
  ]);
  let trailing = format_amount(&d.amount);
  line = align_trailing(line, trailing, comment_col);
  if let Some(comment) = &d.comment {
    line = append_comment(line, &format_comment(comment), config, true);
  }
  writer.write_str(&line);
}

fn format_pad(writer: &mut Writer, d: &ast::Pad<'_>, config: &Configuration) {
  let mut line = join_parts([
    Some(to_part(&d.date)),
    Some("pad".to_string()),
    Some(to_part(&d.account)),
    Some(to_part(&d.from_account)),
  ]);
  if let Some(comment) = &d.comment {
    line = append_comment(line, &format_comment(comment), config, false);
  }
  writer.write_str(&line);
}

fn format_commodity(writer: &mut Writer, d: &ast::Commodity<'_>, config: &Configuration) {
  let comment_col = config.line_width as usize;
  let mut line = join_parts([Some(to_part(&d.date)), Some("commodity".to_string())]);
  line = align_trailing(line, Some(to_part(&d.currency)), comment_col);
  if let Some(comment) = &d.comment {
    line = append_comment(line, &format_comment(comment), config, true);
  }
  writer.write_str(&line);
}

fn format_price(writer: &mut Writer, d: &ast::Price<'_>, config: &Configuration) {
  let comment_col = config.line_width as usize;
  let mut line = join_parts([
    Some(to_part(&d.date)),
    Some("price".to_string()),
    Some(to_part(&d.currency)),
  ]);
  let trailing = format_amount(&d.amount);
  line = align_trailing(line, trailing, comment_col);
  if let Some(comment) = &d.comment {
    line = append_comment(line, &format_comment(comment), config, true);
  }
  writer.write_str(&line);
}

fn format_event(writer: &mut Writer, d: &ast::Event<'_>, config: &Configuration) {
  let mut line = join_parts([
    Some(to_part(&d.date)),
    Some("event".to_string()),
    Some(to_part(&d.event_type)),
    Some(to_part(&d.desc)),
  ]);
  if let Some(comment) = &d.comment {
    line = append_comment(line, &format_comment(comment), config, false);
  }
  writer.write_str(&line);
}

fn format_query(writer: &mut Writer, d: &ast::Query<'_>, config: &Configuration) {
  let mut line = join_parts([
    Some(to_part(&d.date)),
    Some("query".to_string()),
    Some(to_part(&d.name)),
    Some(to_part(&d.query)),
  ]);
  if let Some(comment) = &d.comment {
    line = append_comment(line, &format_comment(comment), config, false);
  }
  writer.write_str(&line);
}

fn format_note(writer: &mut Writer, d: &ast::Note<'_>, config: &Configuration) {
  let mut line = join_parts([
    Some(to_part(&d.date)),
    Some("note".to_string()),
    Some(to_part(&d.account)),
    Some(to_part(&d.note)),
  ]);
  if let Some(comment) = &d.comment {
    line = append_comment(line, &format_comment(comment), config, false);
  }
  writer.write_str(&line);
}

fn format_document(writer: &mut Writer, d: &ast::Document<'_>, config: &Configuration) {
  let mut line = join_parts([
    Some(to_part(&d.date)),
    Some("document".to_string()),
    Some(to_part(&d.account)),
    Some(to_part(&d.filename)),
    d.tags_links.as_ref().map(|t| t.content.trim().to_string()),
  ]);
  if let Some(comment) = &d.comment {
    line = append_comment(line, &format_comment(comment), config, false);
  }
  writer.write_str(&line);
}

fn format_custom(writer: &mut Writer, d: &ast::Custom<'_>, config: &Configuration) {
  let mut line = join_parts([
    Some(to_part(&d.date)),
    Some("custom".to_string()),
    Some(to_part(&d.name)),
    if d.values.is_empty() {
      None
    } else {
      Some(
        d.values
          .iter()
          .map(|v| v.raw.content.trim())
          .collect::<Vec<_>>()
          .join(" "),
      )
    },
  ]);
  if let Some(comment) = &d.comment {
    line = append_comment(line, &format_comment(comment), config, false);
  }
  writer.write_str(&line);
}

fn format_option(writer: &mut Writer, d: &ast::OptionDirective<'_>) {
  let line = join_parts([
    Some("option".to_string()),
    Some(to_part(&d.key)),
    Some(to_part(&d.value)),
  ]);
  writer.write_str(&line);
}

fn format_include(writer: &mut Writer, d: &ast::Include<'_>) {
  let line = join_parts([Some("include".to_string()), Some(to_part(&d.filename))]);
  writer.write_str(&line);
}

fn format_plugin(writer: &mut Writer, d: &ast::Plugin<'_>) {
  let line = join_parts([
    Some("plugin".to_string()),
    Some(to_part(&d.name)),
    d.config.as_ref().map(|c| c.content.trim().to_string()),
  ]);
  writer.write_str(&line);
}

fn format_pushtag(writer: &mut Writer, d: &ast::TagDirective<'_>) {
  let tag = format!("#{}", to_part(&d.tag));
  let line = join_parts([Some("pushtag".to_string()), Some(tag)]);
  writer.write_str(&line);
}

fn format_poptag(writer: &mut Writer, d: &ast::TagDirective<'_>) {
  let tag = format!("#{}", to_part(&d.tag));
  let line = join_parts([Some("poptag".to_string()), Some(tag)]);
  writer.write_str(&line);
}

fn format_pushmeta(writer: &mut Writer, d: &ast::PushMeta<'_>) {
  let key_value = if let Some(value) = d.value.as_ref() {
    format!("{}: {}", d.key.content, value.content.as_str())
  } else {
    format!("{}:", d.key.content)
  };
  let line = join_parts([Some("pushmeta".to_string()), Some(normalize_key_value(&key_value))]);
  writer.write_str(&line);
}

fn format_popmeta(writer: &mut Writer, d: &ast::PopMeta<'_>) {
  let line = join_parts([Some("popmeta".to_string()), Some(format!("{}:", to_part(&d.key)))]);
  writer.write_str(&line);
}

impl Writer {
  fn with_capacity(capacity: usize) -> Self {
    Self {
      buf: String::with_capacity(capacity),
    }
  }

  fn write_str(&mut self, piece: &str) {
    self.buf.push_str(piece);
  }

  fn finish(self) -> String {
    self.buf
  }
}

struct FormatterContext<'a> {
  config: &'a Configuration,
  writer: Writer,
}

impl<'a> FormatterContext<'a> {
  fn new(config: &'a Configuration, capacity: usize) -> Self {
    Self {
      config,
      writer: Writer::with_capacity(capacity),
    }
  }

  fn finish(self) -> String {
    self.writer.finish()
  }

  fn write(&mut self, piece: &str) {
    self.writer.write_str(piece);
  }

  fn format_span(&mut self, span: ast::Span, full_source: &str) {
    let slice = &full_source[span.start..span.end];
    self.write(&normalize_indentation(slice, self.config.indent_width));
    // normalize_indentation already wrote trailing newlines; caller adds newline.
    if self.writer.buf.ends_with('\n') {
      self.writer.buf.pop();
    }
  }

  fn format_directive(&mut self, dir: &Directive<'a>, full_source: &str) {
    match dir {
      Directive::Open(d) => {
        format_open(&mut self.writer, d, self.config);
        self.format_key_values(&d.key_values, full_source);
      }
      Directive::Close(d) => {
        format_close(&mut self.writer, d, self.config);
        self.format_key_values(&d.key_values, full_source);
      }
      Directive::Balance(d) => {
        format_balance(&mut self.writer, d, self.config);
        self.format_key_values(&d.key_values, full_source);
      }
      Directive::Pad(d) => {
        format_pad(&mut self.writer, d, self.config);
        self.format_key_values(&d.key_values, full_source);
      }
      Directive::Transaction(d) => self.format_transaction(d, full_source),
      Directive::Commodity(d) => {
        format_commodity(&mut self.writer, d, self.config);
        self.format_key_values(&d.key_values, full_source);
      }
      Directive::Price(d) => {
        format_price(&mut self.writer, d, self.config);
        self.format_key_values(&d.key_values, full_source);
      }
      Directive::Event(d) => {
        format_event(&mut self.writer, d, self.config);
        self.format_key_values(&d.key_values, full_source);
      }
      Directive::Query(d) => {
        format_query(&mut self.writer, d, self.config);
        self.format_key_values(&d.key_values, full_source);
      }
      Directive::Note(d) => {
        format_note(&mut self.writer, d, self.config);
        self.format_key_values(&d.key_values, full_source);
      }
      Directive::Document(d) => {
        format_document(&mut self.writer, d, self.config);
        self.format_key_values(&d.key_values, full_source);
      }
      Directive::Custom(d) => {
        format_custom(&mut self.writer, d, self.config);
        self.format_key_values(&d.key_values, full_source);
      }
      Directive::Option(d) => format_option(&mut self.writer, d),
      Directive::Include(d) => format_include(&mut self.writer, d),
      Directive::Plugin(d) => format_plugin(&mut self.writer, d),
      Directive::PushTag(d) => format_pushtag(&mut self.writer, d),
      Directive::PopTag(d) => format_poptag(&mut self.writer, d),
      Directive::PushMeta(d) => format_pushmeta(&mut self.writer, d),
      Directive::PopMeta(d) => format_popmeta(&mut self.writer, d),
      Directive::Headline(d) => self.format_span(d.span, full_source),
      Directive::Comment(d) => self.format_span(d.span, full_source),
    }
  }

  fn format_transaction(&mut self, txn: &ast::Transaction<'a>, full_source: &str) {
    let txn_text = &full_source[txn.span.start..txn.span.end];
    let mut lines: Vec<String> = txn_text.replace("\r\n", "\n").lines().map(|l| l.to_string()).collect();

    let mut header_parts: Vec<String> = Vec::new();
    header_parts.push(txn.date.content.trim().to_string());
    if let Some(flag) = &txn.txn {
      header_parts.push(flag.content.trim().to_string());
    }
    if let Some(payee) = &txn.payee {
      header_parts.push(payee.content.trim().to_string());
    }
    if let Some(narration) = &txn.narration {
      header_parts.push(narration.content.trim().to_string());
    }
    if let Some(tags) = &txn.tags_links {
      header_parts.push(tags.content.trim().to_string());
    }
    let mut header_line = header_parts.join(" ");
    if let Some(comment) = &txn.comment {
      header_line = append_comment(header_line, &format_comment(comment), self.config, false);
    }
    lines[0] = header_line;

    let mut posting_line_indices = Vec::new();
    let mut min_indent = usize::MAX;

    for posting in &txn.postings {
      let offset = posting.span.start.saturating_sub(txn.span.start);
      let line_idx = count_newlines_up_to(txn_text, offset);
      posting_line_indices.push(line_idx);
      if let Some(line) = lines.get(line_idx) {
        let indent = leading_indent_width(line, self.config.indent_width);
        min_indent = min_indent.min(indent);
      }
    }

    if min_indent == usize::MAX {
      min_indent = (self.config.indent_width as usize) * 2;
    }

    for (posting, &line_idx) in txn.postings.iter().zip(posting_line_indices.iter()) {
      let flag = posting.opt_flag.as_ref().map(|f| f.content.trim());
      let account = posting.account.content.trim();
      let trailing = if let Some(amount) = posting.amount.as_ref().and_then(format_amount) {
        let mut parts = vec![amount];
        if let Some(cost) = posting.cost_spec.as_ref() {
          parts.push(compact_ws(cost.raw.content));
        }
        if let Some(price_op) = posting.price_operator.as_ref() {
          parts.push(match price_op.content {
            PriceOperator::PerUnit => "@".to_string(),
            PriceOperator::Total => "@@".to_string(),
          });
        }
        if let Some(price_ann) = posting.price_annotation.as_ref() {
          parts.push(compact_ws(price_ann.raw.content));
        }
        Some(parts.join(" "))
      } else {
        None
      };

      let mut line = String::new();
      line.push_str(&" ".repeat(min_indent));
      if let Some(f) = flag {
        line.push_str(f);
        line.push(' ');
      }
      line.push_str(account);

      line = align_trailing(line, trailing, self.config.line_width as usize);

      if let Some(comment) = &posting.comment {
        line = append_comment(line, &format_comment(comment), self.config, true);
      }

      if let Some(slot) = lines.get_mut(line_idx) {
        *slot = line;
      }
    }

    for (idx, line) in lines.iter_mut().enumerate().skip(1) {
      if posting_line_indices.contains(&idx) {
        continue;
      }
      *line = normalize_indentation(line, self.config.indent_width);
    }

    self.write(&lines.join("\n"));
  }

  fn format_key_values(&mut self, key_values: &[ast::KeyValue<'a>], full_source: &str) {
    if key_values.is_empty() {
      return;
    }

    let indent = " ".repeat(self.config.indent_width as usize);

    for kv in key_values {
      self.write("\n");

      let slice = &full_source[kv.span.start..kv.span.end];
      let mut text = normalize_indentation(slice, self.config.indent_width);
      if text.ends_with('\n') {
        text.pop();
      }

      if text.starts_with(char::is_whitespace) {
        self.write(&text);
      } else {
        self.write(&indent);
        self.write(&text);
      }
    }
  }
}

pub fn format(path: Option<&str>, source_text: &str, config: &Configuration) -> Result<String> {
  format_content(path, source_text, config)
}

fn format_content(path: Option<&str>, content: &str, formatting_config: &Configuration) -> Result<String> {
  let path = path.unwrap_or("<memory>");

  if content.trim().is_empty() {
    return Ok(String::new());
  }

  // The parser expects a trailing newline; append one if it's missing.
  let content = if content.ends_with('\n') || content.ends_with("\r\n") {
    content.to_string()
  } else {
    format!("{}\n", content)
  };

  let directives = parse_source(&content, path).map_err(anyhow::Error::new)?;

  let newline = match formatting_config.new_line {
    NewLineKind::LF => "\n",
    NewLineKind::CRLF => "\r\n",
  };

  let mut ctx = FormatterContext::new(formatting_config, content.len());
  let mut prev_end_line: Option<usize> = None;
  let mut prev_is_txn = false;

  for dir in directives.iter() {
    let is_txn = matches!(dir, Directive::Transaction(_));
    if let Some(prev_end) = prev_end_line {
      let start_line = directive_start_line(dir, &content);
      let mut blank_lines = start_line.saturating_sub(prev_end + 1).min(2);
      let txn_min = if (prev_is_txn && !is_txn) || (!prev_is_txn && is_txn) {
        1
      } else {
        0
      };
      if blank_lines < txn_min {
        blank_lines = txn_min;
      }
      for _ in 0..blank_lines {
        ctx.write(newline);
      }
    }

    ctx.format_directive(dir, &content);
    ctx.write(newline);

    prev_end_line = Some(directive_end_line(dir, &content));
    prev_is_txn = is_txn;
  }

  // From this point on we only normalize newline style; the per-node formatter
  // should not add extra trailing newlines beyond what we explicitly wrote.
  let mut formatted = ctx.finish();

  if newline == "\r\n" {
    // Convert lone LF to CRLF, but don't double-convert existing CRLF.
    formatted = formatted.replace("\r\n", "\n");
    formatted = formatted.replace("\n", "\r\n");
  } else {
    // Normalize any CRLF sequences back to LF.
    formatted = formatted.replace("\r\n", "\n");
  }

  // Collapse multiple trailing newlines down to a single newline token.
  let had_trailing_newline = formatted.ends_with(newline);
  formatted = formatted.trim_end_matches(newline).to_string();
  if had_trailing_newline {
    formatted.push_str(newline);
  }

  // Always ensure a single trailing newline for downstream consumers.
  if newline == "\r\n" {
    if !formatted.ends_with("\r\n") {
      formatted.push_str("\r\n");
    }
  } else if !formatted.ends_with('\n') {
    formatted.push('\n');
  }

  Ok(formatted)
}

/// Normalizes tabs to spaces (respecting indent width) outside of string literals and trims trailing whitespace per line.
fn normalize_indentation(text: &str, indent_width: u8) -> String {
  let mut out = String::with_capacity(text.len());

  for (i, line) in text.replace("\r\n", "\n").lines().enumerate() {
    if i > 0 {
      out.push('\n');
    }

    // Expand tabs outside of string literals, then trim trailing whitespace.
    let expanded = expand_tabs_outside_strings(line, indent_width);
    let trimmed = expanded.trim_end();
    out.push_str(trimmed);
  }

  out
}

/// Expand tabs to spaces while skipping tabs that appear inside string literals.
/// Leading tabs expand to the configured indent width; tabs elsewhere become a single space.
fn expand_tabs_outside_strings(line: &str, indent_width: u8) -> String {
  let indent = " ".repeat(indent_width as usize);
  let mut out = String::with_capacity(line.len());
  let mut in_string = false;
  let mut escape = false;
  let mut at_line_start = true;

  for ch in line.chars() {
    if in_string {
      out.push(ch);
      if escape {
        escape = false;
        continue;
      }
      match ch {
        '\\' => escape = true,
        '"' => in_string = false,
        _ => {}
      }
      at_line_start = false;
      continue;
    }

    match ch {
      '"' => {
        in_string = true;
        out.push(ch);
        at_line_start = false;
      }
      '\t' => {
        if at_line_start {
          out.push_str(&indent);
        } else {
          out.push(' ');
        }
      }
      _ => {
        out.push(ch);
        at_line_start = false;
      }
    }
  }

  out
}

fn count_newlines_up_to(text: &str, offset: usize) -> usize {
  text
    .as_bytes()
    .iter()
    .take(offset.min(text.len()))
    .filter(|b| **b == b'\n')
    .count()
}

fn directive_span(dir: &Directive<'_>) -> ast::Span {
  match dir {
    Directive::Open(d) => d.span,
    Directive::Close(d) => d.span,
    Directive::Balance(d) => d.span,
    Directive::Pad(d) => d.span,
    Directive::Transaction(d) => d.span,
    Directive::Commodity(d) => d.span,
    Directive::Price(d) => d.span,
    Directive::Event(d) => d.span,
    Directive::Query(d) => d.span,
    Directive::Note(d) => d.span,
    Directive::Document(d) => d.span,
    Directive::Custom(d) => d.span,
    Directive::Option(d) => d.span,
    Directive::Include(d) => d.span,
    Directive::Plugin(d) => d.span,
    Directive::PushTag(d) => d.span,
    Directive::PopTag(d) => d.span,
    Directive::PushMeta(d) => d.span,
    Directive::PopMeta(d) => d.span,
    Directive::Headline(d) => d.span,
    Directive::Comment(d) => d.span,
  }
}

fn line_at_offset(text: &str, offset: usize) -> usize {
  count_newlines_up_to(text, offset) + 1
}

fn directive_start_line(dir: &Directive<'_>, text: &str) -> usize {
  let span = directive_span(dir);
  line_at_offset(text, span.start)
}

fn directive_end_line(dir: &Directive<'_>, text: &str) -> usize {
  let span = directive_span(dir);
  let end_offset = span.end.saturating_sub(1);
  line_at_offset(text, end_offset)
}

fn leading_indent_width(line: &str, indent_width: u8) -> usize {
  let mut width = 0usize;
  for ch in line.chars() {
    match ch {
      ' ' => width += 1,
      '\t' => width += indent_width as usize,
      _ => break,
    }
  }
  width
}

fn join_parts(parts: impl IntoIterator<Item = Option<String>>) -> String {
  let mut out = Vec::new();
  for p in parts.into_iter().flatten() {
    if !p.is_empty() {
      out.push(p);
    }
  }
  out.join(" ")
}

fn to_part(text: &WithSpan<&str>) -> String {
  text.content.trim().to_string()
}

fn compact_ws(text: &str) -> String {
  text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_key_value(text: &str) -> String {
  let mut parts = text.splitn(2, ':');
  let key = parts.next().unwrap_or("").trim();
  let value = parts.next().unwrap_or("").trim();
  if value.is_empty() {
    format!("{}:", key)
  } else {
    format!("{}: {}", key, value)
  }
}

fn append_comment(mut line: String, comment: &str, config: &Configuration, align: bool) -> String {
  let trimmed = line.trim_end().to_string();
  let base_len = trimmed.len();
  let target = config.line_width as usize;

  line = trimmed;
  if align && base_len < target {
    line.push_str(&" ".repeat(target - base_len));
  } else if !line.ends_with(' ') {
    line.push(' ');
  }

  line.push_str(comment);
  line
}

fn align_trailing(mut base: String, trailing: Option<String>, comment_col: usize) -> String {
  if let Some(value) = trailing {
    let value_len = value.len();
    let target_end = comment_col.saturating_sub(2);
    let desired_start = target_end.saturating_sub(value_len.saturating_sub(1));
    let start = desired_start.max(base.len().saturating_add(1));

    if base.len() < start {
      base.push_str(&" ".repeat(start - base.len()));
    }
    base.push_str(&value);
  }

  base
}

fn format_amount(amount: &ast::Amount<'_>) -> Option<String> {
  if let Some(currency) = &amount.currency {
    let raw = amount.raw.content;
    let start = currency.span.start.saturating_sub(amount.raw.span.start);
    if start <= raw.len() {
      let number = compact_ws(&raw[..start]);
      let cur = currency.content.trim();
      if !number.is_empty() && !cur.is_empty() {
        return Some(format!("{} {}", number, cur));
      }
    }
  }

  Some(compact_ws(amount.raw.content))
}

fn format_currencies(currencies: &[WithSpan<&str>]) -> Option<String> {
  if currencies.is_empty() {
    return None;
  }
  Some(
    currencies
      .iter()
      .map(|c| c.content.trim())
      .collect::<Vec<_>>()
      .join(" "),
  )
}

fn format_comment(raw: &WithSpan<&str>) -> String {
  let trimmed = raw.content.trim();
  let without_semicolon = trimmed.strip_prefix(';').unwrap_or(trimmed).trim_start();
  if without_semicolon.is_empty() {
    ";".to_string()
  } else {
    format!("; {}", without_semicolon)
  }
}
