#!/usr/bin/env xiom
// XIOM Markdown → HTML Converter
// Usage: xiom run md_to_html.xi  (reads input.md, writes output.html)

use xiom.io;
use xiom.string;

fn process_line(line: Str) -> Str {
  var t = string.str_trim(line);
  var len = string.str_len(t);
  
  if len == 0 { return "<br>"; }
  if t == "---" || t == "***" { return "<hr>"; }
  
  if string.str_starts_with(t, "### ") { return "<h3>" + string.str_slice(t, 4, len) + "</h3>"; }
  if string.str_starts_with(t, "## ") { return "<h2>" + string.str_slice(t, 3, len) + "</h2>"; }
  if string.str_starts_with(t, "# ") { return "<h1>" + string.str_slice(t, 2, len) + "</h1>"; }
  if string.str_starts_with(t, "- ") { return "<li>" + string.str_slice(t, 2, len) + "</li>"; }
  
  return "<p>" + t + "</p>";
}

fn main() {
  var md = io.read_file("input.md");
  if md.is_err { io.println("Error: reading input.md"); return; }

  var lines = string.lines(md.unwrap());
  var result = "<!DOCTYPE html>\n<html>\n<body>\n";
  
  var i = 0;
  var count = lines.len();
  while i < count {
    result = result + process_line(lines[i]) + "\n";
    i = i + 1;
  }
  
  result = result + "</body>\n</html>\n";
  
  if io.write_file("output.html", result).is_err {
    io.println("Error: writing output.html");
    return;
  }
  io.println("Converted input.md -> output.html");
}
