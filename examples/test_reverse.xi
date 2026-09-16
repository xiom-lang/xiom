// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

type Person = {
  name: Str;
  age: Int;
  address: Address;
} derive[Clone]

type Address = {
  street: Str;
  city: Str;
  zip: Int;
} derive[Clone]

fn main() -> Int {
  var addr = Address{ street: "Main", city: "NYC", zip: 10001 };
  var person = Person{ name: "Alice", age: 30, address: addr };
  return 0;
}
