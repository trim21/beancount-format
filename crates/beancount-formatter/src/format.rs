use anyhow::{Context, Result};
use tree_sitter::{Node, Parser, Point};

use crate::ast::Directive;
use crate::configuration::{Configuration, NewLineKind};
use crate::parse::parse_directives;

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
  pending_space: bool,
  last_was_newline: bool,
}

impl<'a> FormatterContext<'a> {
  fn new(config: &'a Configuration, capacity: usize) -> Self {
    Self {
      config,
      writer: Writer::with_capacity(capacity),
      pending_space: false,
      last_was_newline: true,
    }
  }

  fn finish(self) -> String {
    self.writer.finish()
  }

  fn write(&mut self, piece: &str) {
    self.writer.write_str(piece);
  }

  fn format_span(&mut self, span: crate::ast::Span, full_source: &str) {
    let slice = &full_source[span.start..span.end];
    self.write(&normalize_indentation(slice, self.config.indent_width));
    // normalize_indentation already wrote trailing newlines; caller adds newline.
    if self.writer.buf.ends_with('\n') {
      self.writer.buf.pop();
    }
  }

  fn format_directive(&mut self, dir: &Directive<'a>, full_source: &str) {
    match dir {
      Directive::Open(d) => self.format_span(d.span, full_source),
      Directive::Close(d) => self.format_span(d.span, full_source),
      Directive::Balance(d) => self.format_span(d.span, full_source),
      Directive::Pad(d) => self.format_span(d.span, full_source),
      Directive::Transaction(d) => self.format_span(d.span, full_source),
      Directive::Commodity(d) => self.format_span(d.span, full_source),
      Directive::Price(d) => self.format_span(d.span, full_source),
      Directive::Event(d) => self.format_span(d.span, full_source),
      Directive::Query(d) => self.format_span(d.span, full_source),
      Directive::Note(d) => self.format_span(d.span, full_source),
      Directive::Document(d) => self.format_span(d.span, full_source),
      Directive::Custom(d) => self.format_span(d.span, full_source),
      Directive::Option(d) => self.format_span(d.span, full_source),
      Directive::Include(d) => self.format_span(d.span, full_source),
      Directive::Plugin(d) => self.format_span(d.span, full_source),
      Directive::Pushtag(d) => self.format_span(d.span, full_source),
      Directive::Poptag(d) => self.format_span(d.span, full_source),
      Directive::Pushmeta(d) => self.format_span(d.span, full_source),
      Directive::Popmeta(d) => self.format_span(d.span, full_source),
      Directive::Raw(d) => self.format_span(d.span, full_source),
    }
  }

  // AST-driven formatter helpers (currently unused by the stable formatting path).
  // Keep them around for incremental migration without triggering dead_code warnings.
  fn newline_lf(&mut self) {
    self.pending_space = false;
    if !self.last_was_newline {
      self.write("\n");
      self.last_was_newline = true;
    }
  }

  fn space(&mut self) {
    if !self.last_was_newline {
      self.pending_space = true;
    }
  }

  fn write_token(&mut self, token: &str) {
    if self.pending_space {
      self.write(" ");
      self.pending_space = false;
    }
    self.write(token);
    self.last_was_newline = token.ends_with('\n');
    if self.last_was_newline {
      self.pending_space = false;
    }
  }

  /// AST-driven formatter: format the given node by walking its children.
  ///
  /// Contract (current):
  /// - Preserve token order.
  /// - Convert tabs to spaces (outside string literals).
  /// - Trim trailing whitespace per line.
  /// - Avoid producing extra blank lines.
  fn format_node(&mut self, node: Node, text: &str) {
    if node.child_count() == 0 {
      self.format_leaf_token(slice_text(node, text));
      return;
    }

    // We need to preserve separators (spaces, newlines, punctuation) that are not
    // represented as named nodes. Tree-sitter stores them as part of the source
    // between children byte ranges.
    let mut cursor = node.walk();
    let mut prev_end = node.start_byte();
    let named: Vec<Node> = node.named_children(&mut cursor).collect();

    for child in named.iter().copied() {
      if child.is_missing() || child.is_error() {
        continue;
      }

      // Write interleaving raw text between children, normalized.
      let gap = &text[prev_end..child.start_byte()];
      self.format_gap_text(gap);
      // If the gap had only horizontal whitespace, ensure we keep exactly one
      // separating space between tokens.
      let gap_norm = gap.replace("\r\n", "\n");
      if !gap_norm.contains('\n') && gap_norm.chars().all(|c| c == ' ' || c == '\t') && !self.last_was_newline {
        self.pending_space = true;
      }

      self.format_node(child, text);
      prev_end = child.end_byte();
    }

    // Tail gap after last child.
    let gap = &text[prev_end..node.end_byte()];
    self.format_gap_text(gap);
  }

  /// Formats a leaf *token* text (already a concrete grammar token), preserving
  /// its internal spaces but normalizing tabs and trimming trailing whitespace.
  fn format_leaf_token(&mut self, text: &str) {
    for (i, line) in text.replace("\r\n", "\n").split('\n').enumerate() {
      if i > 0 {
        self.newline_lf();
      }

      if line.is_empty() {
        continue;
      }

      let expanded = expand_tabs_outside_strings(line, self.config.indent_width);
      let trimmed = expanded.trim_end();
      if trimmed.is_empty() {
        continue;
      }
      self.write_token(trimmed);
    }
  }

  /// Formats the raw text between AST nodes. This is where we normalize
  /// separators: collapse horizontal whitespace to a single space, preserve
  /// newlines, and trim spaces before newlines.
  fn format_gap_text(&mut self, text: &str) {
    let text = text.replace("\r\n", "\n");
    let mut seen_space = false;

    for ch in text.chars() {
      match ch {
        '\n' => {
          seen_space = false;
          self.newline_lf();
        }
        ' ' | '\t' => {
          if !self.last_was_newline {
            seen_space = true;
          }
        }
        _ => {
          if seen_space {
            self.pending_space = true;
            seen_space = false;
          }
          let mut buf = [0u8; 4];
          let s = ch.encode_utf8(&mut buf);
          self.write_token(s);
        }
      }
    }
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

  let directives = parse_directives(root, &content, path.to_string()).map_err(anyhow::Error::new)?;

  let newline = match formatting_config.new_line {
    NewLineKind::LF => "\n",
    NewLineKind::CRLF => "\r\n",
  };

  let mut ctx = FormatterContext::new(formatting_config, content.len());
  for dir in &directives {
    ctx.format_directive(dir, &content);
    ctx.write(newline);
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
