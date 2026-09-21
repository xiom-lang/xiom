// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A17: Array of enum -- enum values passed through struct wrapper fields
enum Color { Red, Green, Blue }
fn color_val(c: Color) -> Int {
  match c { Red => 1, Green => 2, Blue => 3, }
}
type Entry = { kind: Color; }
fn main() -> Int {
  var items = [Entry{ kind: Color.Red; }, Entry{ kind: Color.Green; }, Entry{ kind: Color.Blue; }, Entry{ kind: Color.Red; }, Entry{ kind: Color.Blue; }];
  var sum: Int = 0;
  var i: Int = 0;
  while i < 5 {
    sum += color_val(items[i].kind);
    i += 1;
  }
  if sum == 10 { return 0; }
  return 1;
}
