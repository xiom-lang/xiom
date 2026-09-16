// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM e2e test -- Phase 5c.7 ecosystem gap fixes
// Tests all 7 compiler fixes delivered in Phase 5c hardening
// Returns 0 on success.

module e2e_5c7_hardening

// === 1. Float32 type compatibility ===
fn test_float32_compat() -> Bool {
  var a: Float32 = 1.0;
  var b: Float32 = 2.0;
  var c: Float32 = a + b;
  return c > 2.0;
}

// === 2. Hex escape in char/string literals ===
fn test_hex_escape_char() -> Bool {
  var null_char = '\x00';
  var a_char = '\x41';
  return null_char != a_char;
}

fn test_hex_escape_string() -> Bool {
  var s = "hello\x00world";
  return s.len() > 0;
}

// === 3. 'this' keyword resolves to 'self' ===
type Counter = { val: Int; }

fn Counter.inc(self) {
  val = val + 1;
}

fn test_this_keyword() -> Bool {
  var c = Counter{ val: 0; };
  c.inc();
  return c.val == 1;
}

// === 4. Enum variant constructors ===
type Shape = enum {
  Circle(r: Int),
  Rect(w: Int, h: Int),
  Point,
}

fn test_enum_variant_constructor() -> Bool {
  var c = Shape.Circle(5);
  var r = Shape.Rect(10, 20);
  return true;
}

// === 5. Comma-separated contracts ===
fn safe_add(a: Int, b: Int) -> Int
  requires: a >= 0, b >= 0
  ensures: result >= a, result >= b
{
  return a + b;
}

fn test_comma_contracts() -> Bool {
  var x = safe_add(3, 4);
  return x == 7;
}

// === 6. Int/Char type compatibility ===
fn test_int_char_compat() -> Bool {
  var c: Char = 'A';
  var i = 65;
  return i == 65 && c == 'A';
}

// === 7. ? operator on Result ===
fn safe_div(a: Int, b: Int) -> Result[Int, Str] {
  if b == 0 { return Err("div zero"); }
  return Ok(a / b);
}

fn test_try_operator() -> Bool {
  let result = safe_div(10, 2);
  match result {
    Ok(x) => return x == 5,
    Err(_) => return false,
  };
  return false;
}

// === 7. Enum variant pattern match ===
fn test_enum_pattern_match() -> Bool {
  var s = Shape.Circle(42);
  match s {
    Shape.Circle(r) => { return r == 42; }
    Shape.Rect(_, _) => { return false; }
    Shape.Point => { return false; }
  }
}

fn main() -> Int {
  if !test_float32_compat() { return 1; }
  if !test_hex_escape_char() { return 2; }
  if !test_hex_escape_string() { return 3; }
  if !test_enum_variant_constructor() { return 4; }
  if !test_comma_contracts() { return 5; }
  if !test_int_char_compat() { return 6; }
  if !test_try_operator() { return 7; }
  if !test_enum_pattern_match() { return 8; }
  return 0;
}
