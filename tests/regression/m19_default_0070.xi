// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0070

interface Greeter {
  fn greet(&self) -> Str { return "hello"; }
}

type Dog = {}

fn Dog.greet(&self) -> Str { return "woof"; }

type Cat = {}

fn main() -> Int {
  var d: Dog = Dog{};
  var c: Cat = Cat{};
  if d.greet() == "woof" && c.greet() == "hello" { return 0; }
  return 1;
}
