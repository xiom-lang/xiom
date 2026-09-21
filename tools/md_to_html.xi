#!/usr/bin/env xiom
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// MD->HTML -- v0.52.7+ with String Copy semantics

use xiom.io;
use xiom.string;

fn main() -> Int {
  var md_result = io.read_file("input.md");
  match md_result {
    Ok(md) => {
      var lines = string.lines(md);
      var result = "<!DOCTYPE html>\n<html>\n<body>\n";
      var i = 0;

      while i < lines.len() {
        var line = string.str_trim(lines[i]);

        if line == "" { result = result + "<br>\n"; i = i + 1; continue; }
        if line == "---" { result = result + "<hr>\n"; i = i + 1; continue; }

        var p3 = string.str_slice(line, 0, 4);
        var p2 = string.str_slice(line, 0, 3);
        var p1 = string.str_slice(line, 0, 2);
        var lp = string.str_slice(line, 0, 2);

        if p3 == "### " {
          result = result + "<h3>" + string.str_slice(line, 4, string.str_len(line)) + "</h3>\n";
        } elif p2 == "## " {
          result = result + "<h2>" + string.str_slice(line, 3, string.str_len(line)) + "</h2>\n";
        } elif p1 == "# " {
          result = result + "<h1>" + string.str_slice(line, 2, string.str_len(line)) + "</h1>\n";
        } elif lp == "- " {
          result = result + "<li>" + string.str_slice(line, 2, string.str_len(line)) + "</li>\n";
        } else {
          result = result + "<p>" + line + "</p>\n";
        }
        i = i + 1;
      }

      result = result + "</body>\n</html>\n";
      var _ = io.write_file("output.html", result);
      io.println("Done: output.html");
      return 0;
    }
    Err(e) => {
      io.println("ERROR: cannot read input.md: " + e.message);
      return 1;
    }
  }
}
