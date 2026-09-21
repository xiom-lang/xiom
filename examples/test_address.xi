// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

type Address = {
  street: Str;
  city: Str;
  zip: Int;
} derive[Eq, Clone]

type Person = {
  name: Str;
  age: Int;
  address: Address;
} derive[Eq, Clone]

fn main() -> Int {
  var addr = Address{ street: "Main", city: "NYC", zip: 10001 };
  var person = Person{ name: "Alice", age: 30, address: addr };
  var cloned = person.clone();
  
  var addr1 = Address{ street: "A", city: "B", zip: 1 };
  var addr2 = Address{ street: "C", city: "D", zip: 2 };
  var addr3 = Address{ street: "A", city: "B", zip: 1 };
  var addr1c = addr1.clone();
  var addr2c = addr2.clone();
  
  if addr1 == addr3 { return 1; }
  if addr1 == addr1c { return 2; }
  if addr2 == addr2c { return 3; }
  if addr1 == addr2 { return 4; }
  
  var p1 = Person{ name: "Alice", age: 30, address: addr1 };
  var p1c = p1.clone();
  if p1c.address.zip == 10001 { return 5; }
  
  return 0;
}
