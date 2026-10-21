// m1731 (REL-20261021-1731-stdlib): `match &enum` must lower like an owned
// match: load the tag through the borrow and bind payloads by value.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Before the fix the variant check compared the alloca POINTER to 0
// (`icmp eq i64 %alloca, 0`, invalid IR) and the String arm never matched.
// The borrowed Str payload must also bind the string, not pointer bits.

module m1731_borrowed_enum_match

use xiom.io;
use xiom.serialize.json;

fn main() -> Int {
  let s = json.json_string("borrowed-world");
  match &s {
    JsonValue.String(v) => {
      io.println("s=" + v);
      if v != "borrowed-world" { return 1; }
    }
    _ => { io.println("string arm missed"); return 2; }
  }

  let n = json.json_number(42.5);
  match &n {
    JsonValue.Number(x) => {
      if x != 42.5 { return 3; }
    }
    _ => { return 4; }
  }

  let b = json.json_bool(true);
  match &b {
    JsonValue.Bool(x) => {
      if x != true { return 5; }
    }
    _ => { return 6; }
  }

  io.println("m1731 ok");
  return 0;
}
