use anyhow::{Context, Result};
use tree_sitter::{Node, Parser, Point};

use crate::configuration::{Configuration, NewLineKind};

/// Simple string writer to avoid building large intermediate vectors before concatenation.
struct Writer {
  buf: String,
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

  /// Formats a transaction entry.
  fn format_transaction(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_balance(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_open(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_close(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_pad(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_document(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_note(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_event(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_price(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_commodity(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_query(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_custom(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_option(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_include(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_plugin(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_pushtag(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_poptag(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_pushmeta(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_popmeta(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  fn format_fallback(&mut self, node: Node, text: &str) {
    self.format_leaf(node, text);
  }

  /// Basic leaf formatter: slices the source for the node, normalizes indentation and trailing whitespace, and enforces LF before newline-kind conversion.
  fn format_leaf(&mut self, node: Node, text: &str) {
    let slice = slice_text(node, text);
    self.write(&normalize_indentation(slice, self.config.indent_width));
  }
}

pub fn format(path: Option<&str>, source_text: &str, config: &Configuration) -> Result<String> {
  format_content(path, source_text, config)
}

fn format_content(path: Option<&str>, content: &str, formatting_config: &Configuration) -> Result<String> {
  let path = path.unwrap_or("<memory>");

  // The parser expects a trailing newline; append one if it's missing.
  let content = if content.ends_with('\n') || content.ends_with("\r\n") {
    content.to_string()
  } else {
    format!("{}\n", content)
  };

  let mut parser = Parser::new();

  parser
    .set_language(&tree_sitter_beancount::language())
    .context("Failed to load beancount grammar")?;

  let tree = parser
    .parse(&content, None)
    .ok_or_else(|| anyhow::anyhow!("Failed to parse {}", path))?;

  if tree.root_node().has_error() {
    let error_message = describe_parse_errors(tree.root_node(), &content);
    return Err(anyhow::anyhow!("Failed to parse {}: {}", path, error_message));
  }

  let root = tree.root_node();

  // Walk the AST and format each top-level declaration/entry via dedicated handlers.
  let mut cursor = root.walk();
  let mut ctx = FormatterContext::new(formatting_config, content.len());
  for node in root.named_children(&mut cursor) {
    match node.kind() {
      "transaction" => ctx.format_transaction(node, &content),
      "balance" => ctx.format_balance(node, &content),
      "open" => ctx.format_open(node, &content),
      "close" => ctx.format_close(node, &content),
      "pad" => ctx.format_pad(node, &content),
      "document" => ctx.format_document(node, &content),
      "note" => ctx.format_note(node, &content),
      "event" => ctx.format_event(node, &content),
      "price" => ctx.format_price(node, &content),
      "commodity" => ctx.format_commodity(node, &content),
      "query" => ctx.format_query(node, &content),
      "custom" => ctx.format_custom(node, &content),
      "option" => ctx.format_option(node, &content),
      "include" => ctx.format_include(node, &content),
      "plugin" => ctx.format_plugin(node, &content),
      "pushtag" => ctx.format_pushtag(node, &content),
      "poptag" => ctx.format_poptag(node, &content),
      "pushmeta" => ctx.format_pushmeta(node, &content),
      "popmeta" => ctx.format_popmeta(node, &content),
      _ => ctx.format_fallback(node, &content),
    }
  }

  let mut formatted = ctx.finish();

  // Always ensure a trailing newline for downstream consumers.
  if !formatted.ends_with('\n') && !formatted.ends_with("\r\n") {
    formatted.push('\n');
  }

  let newline = match formatting_config.new_line_kind {
    NewLineKind::LF => "\n",
    NewLineKind::CRLF => "\r\n",
  };

  if newline == "\r\n" {
    formatted = formatted.replace("\n", "\r\n");
    if !formatted.ends_with("\r\n") {
      formatted.push_str("\r\n");
    }
  } else {
    // Normalize any stray CRLF sequences back to LF and guarantee trailing LF.
    formatted = formatted.replace("\r\n", "\n");
    if !formatted.ends_with('\n') {
      formatted.push('\n');
    }
  }

  Ok(formatted)
}

/// Build a concise error summary from tree-sitter error nodes, including row/col info.
fn describe_parse_errors(root: Node, text: &str) -> String {
  let mut messages = Vec::new();
  let mut stack = vec![root];

  while let Some(node) = stack.pop() {
    if node.is_error() || node.is_missing() {
      let span = format_point_range(node.start_position(), node.end_position());
      let snippet = slice_text(node, text).trim();
      if node.is_missing() {
        messages.push(format!("missing {:?} at {}", node.kind(), span));
      } else {
        messages.push(format!("error at {} near {:?}", span, snippet));
      }
    }

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
      loop {
        stack.push(cursor.node());
        if !cursor.goto_next_sibling() {
          break;
        }
      }
    }
  }

  if messages.is_empty() {
    "unknown parse error".to_string()
  } else {
    messages.join("; ")
  }
}

fn format_point_range(start: Point, end: Point) -> String {
  if start == end {
    format!("{}:{}", start.row + 1, start.column + 1)
  } else {
    format!(
      "{}:{}-{}:{}",
      start.row + 1,
      start.column + 1,
      end.row + 1,
      end.column + 1
    )
  }
}

fn slice_text<'a>(node: Node, text: &'a str) -> &'a str {
  let start = node.start_byte();
  let end = node.end_byte();
  &text[start..end]
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

  // Ensure a single trailing newline for leaf nodes to ease concatenation.
  if !out.ends_with('\n') {
    out.push('\n');
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
