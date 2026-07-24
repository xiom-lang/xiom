#!/usr/bin/env xiom
// XIOM Script — Markdown to HTML Converter (M10 scripting test)
// Usage: cat input.md | xiom run tools/md_to_html.xi
// Demonstrates: stdlib usage, string manipulation, Result handling, implicit main

use xiom.string;
use xiom.io;

fn main() {
  loop {
    var line_result = io.read_line();
    // read_line returns Result — handle EOF gracefully
    match line_result {
      Ok(line) => {
        if string.is_empty(line) { return; }
        process_line(line);
      },
      Err(_) => { return; },
    }
  }
}

fn process_line(line: Str) {
  if string.str_starts_with(line, "```") {
    io.println("<pre><code></code></pre>");
  }
  elif string.str_starts_with(line, "# ") {
    var rest = string.str_slice(line, 2, string.str_len(line));
    io.print("<h1>");
    io.print(rest);
    io.println("</h1>");
  }
  elif string.str_starts_with(line, "## ") {
    var rest = string.str_slice(line, 3, string.str_len(line));
    io.print("<h2>");
    io.print(rest);
    io.println("</h2>");
  }
  elif string.str_starts_with(line, "### ") {
    var rest = string.str_slice(line, 4, string.str_len(line));
    io.print("<h3>");
    io.print(rest);
    io.println("</h3>");
  }
  elif line == "---" {
    io.println("<hr>");
  }
  elif string.str_starts_with(line, "> ") {
    io.print("<blockquote><p>");
    io.print(string.str_slice(line, 2, string.str_len(line)));
    io.println("</p></blockquote>");
  }
  elif string.str_starts_with(line, "- ") {
    io.print("<li>");
    io.print(string.str_slice(line, 2, string.str_len(line)));
    io.println("</li>");
  }
  elif line == "" {
    io.println("<br>");
  }
  else {
    io.print("<p>");
    io.print(line);
    io.println("</p>");
  }
}
