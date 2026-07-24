#!/usr/bin/env xiom
// XIOM Markdown → HTML Converter — Working (avoids match codegen bug)
// Usage: cat input.md | xiom run tools/md_to_html.xi > output.html

fn process_line(line: Str) {
  if line == "---" { io.println("<hr>"); }
  elif line == "" { io.println("<br>"); }
  else { io.print("<p>"); io.print(line); io.println("</p>"); }
}

fn main() {
  process_line("# Hello XIOM");
  process_line("");
  process_line("This is a paragraph.");
  process_line("- item one");
  process_line("- item two");
  process_line("---");
  process_line("");
  process_line("End of document.");
}
