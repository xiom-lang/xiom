// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module tests.ecosystem.test_fnptr

fn add_one() -> Int { return 1; }

fn main() -> Int {
  var v: Vec[fn() -> Int] = Vec[fn() -> Int].new();
  v.push(add_one);
  var f: fn() -> Int = v[0];
  return f();
}