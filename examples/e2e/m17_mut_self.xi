// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M17: &mut self regression test
type Counter = { count: Int; }
fn Counter.inc(&mut self) { count = count + 1; }
fn Counter.add(&mut self, x: Int) { count = count + x; }

type Buf = { data: Str; }
fn Buf.append(&mut self, s: Str) { data = data + s; }

fn test_inc() -> Bool {
  var c = Counter { count: 0; };
  c.inc();
  return c.count == 1;
}

fn test_add() -> Bool {
  var c = Counter { count: 10; };
  c.add(5);
  return c.count == 15;
}

fn test_append() -> Bool {
  var b = Buf { data: "hello"; };
  b.append(" world");
  return b.data == "hello world";
}

fn main() -> Int {
  if !test_inc() { return 1; }
  if !test_add() { return 2; }
  if !test_append() { return 3; }
  return 0;
}
