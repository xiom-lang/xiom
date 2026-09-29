// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m163 lock (2026-09-29): a Vec[Str] STRUCT-FIELD element inside a struct
// METHOD must take the Str-handle load + @xiom_str_concat path. Before the
// fix, `lines[0] + lines[1]` in `Buf.pair` lowered to i64 add + inttoptr
// (silent pointer-decimal garbage / access violation) and the `Buf.join`
// accumulator printed pointer decimals. Root cause: the method prologue
// bound receiver fields as locals but never registered their Vec element
// types, so field Vecs did not behave like local Vecs. See
// docs/COMPILER_BUGS.md "2026-09-29 -- m163 FIXED".
module m163_method_field_vec_concat

type Buf = {
  lines: Vec[Str];
}

fn Buf.add(text: Str) {
  lines.push(text);
}

fn Buf.pair() -> Str {
  return lines[0] + lines[1];
}

fn Buf.join() -> Str {
  var out = "";
  var i = 0;
  while i < lines.len() {
    out = out + lines[i] + "\n";
    i = i + 1;
  }
  return out;
}

fn main() -> Int {
  var b = Buf{ lines: Vec[Str].new() };
  b.add("AAA");
  b.add("BBB");

  // Direct field read (worked before; must keep working).
  if b.lines[0] != "AAA" { return 1; }

  // Method-context field-element concat (m163; was int add + inttoptr).
  if b.pair() != "AAABBB" { return 2; }

  // Method-context accumulator over the field (m163; was pointer decimals).
  if b.join() != "AAA\nBBB\n" { return 3; }

  // Local-Vec accumulation guard (worked before; must keep working).
  var v = Vec[Str].new();
  v.push("XX");
  v.push("YY");
  var s = "";
  var i = 0;
  while i < v.len() {
    s = s + v[i] + "\n";
    i = i + 1;
  }
  if s != "XX\nYY\n" { return 4; }

  return 0;
}
