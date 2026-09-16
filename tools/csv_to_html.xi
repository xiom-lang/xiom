#!/usr/bin/env xiom
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM CSV -> HTML Converter -- Gap Discovery Script
// Usage: xiom run tools/csv_to_html.xi
// Tests: file I/O, string splitting, Vec ops, loops, HTML generation
// Note: reads tools/_sample.csv via absolute path (G12: relative paths resolve from temp dir)

fn split_csv_line(line: Str) -> Vec[Str] {
  var result: Vec[Str] = Vec[Str]::new();
  var field: Str = "";
  var in_quotes = false;
  var i: Int = 0;
  let line_len = line.len();
  while i < line_len {
    var c = line.byte_at(i);
    if c == 34 {
      if in_quotes && i + 1 < line_len && line.byte_at(i + 1) == 34 {
        field = field + "\"";
        i = i + 1;
      } else {
        in_quotes = !in_quotes;
      };
    } elif c == 44 && !in_quotes {
      result.push(field);
      field = "";
    } else {
      field = field + line.slice(i, i + 1);
    };
    i = i + 1;
  };
  result.push(field);
  return result;
}

fn escape_html(s: Str) -> Str {
  var result: Str = "";
  var i: Int = 0;
  let slen = s.len();
  while i < slen {
    var c = s.byte_at(i);
    if c == 60 {
      result = result + "&lt;";
    } elif c == 62 {
      result = result + "&gt;";
    } elif c == 38 {
      result = result + "&amp;";
    } elif c == 34 {
      result = result + "&quot;";
    } else {
      result = result + s.slice(i, i + 1);
    };
    i = i + 1;
  };
  return result;
}

fn main() {
  var result = io.read_file("tools/_sample.csv");
  var csv_text: Str;
  match result {
    Ok(s) => { csv_text = s; };
    Err(e) => { io.println("error: " + e.message); return; };
  };

  var lines: Vec[Str] = Vec[Str]::new();
  var line_start: Int = 0;
  var pos: Int = 0;
  let text_len = csv_text.len();
  while pos < text_len {
    if csv_text.byte_at(pos) == 10 {
      lines.push(csv_text.slice(line_start, pos));
      line_start = pos + 1;
    };
    pos = pos + 1;
  };
  if line_start < text_len {
    lines.push(csv_text.slice(line_start, text_len));
  };

  let nlines = lines.len();
  if nlines == 0 {
    io.println("<p>No data</p>");
    return;
  };

  io.println("<!DOCTYPE html>");
  io.println("<html><head><title>CSV</title>");
  io.println("<style>table{border-collapse:collapse}th,td{border:1px solid #ccc;padding:4px 8px}</style>");
  io.println("</head><body>");
  io.println("<table>");

  var first = true;
  var row_idx: Int = 0;
  while row_idx < nlines {
    var fields = split_csv_line(lines[row_idx]);
    io.println("<tr>");
    var col: Int = 0;
    let nfields = fields.len();
    while col < nfields {
      if first {
        io.print("<th>" + escape_html(fields[col]) + "</th>");
      } else {
        io.print("<td>" + escape_html(fields[col]) + "</td>");
      };
      col = col + 1;
    };
    io.println("</tr>");
    if first { first = false; };
    row_idx = row_idx + 1;
  };

  io.println("</table>");
  io.println("</body></html>");
}

