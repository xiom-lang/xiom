// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m221 (XVC-C-06): enums whose payload field names COLLIDE across variants
// (IntVal(v: Int), FloatVal(v: Float32), ...) erase the shared slot to i64
// (M19) and readers decode per enum_variant_field_types -- so the ctor must
// pack the DECLARED width's bits. Storing the raw double bits of 2.5 made
// `FloatVal(2.5)` extract as 0.0 (the low 32 bits of the f64 pattern are
// zero), build/unit-dependent. Float64 and Str payloads must be unchanged.

module m221_enum_f32_payload_pack

use xiom.num.float;

enum Value {
  IntVal(v: Int),
  FloatVal(v: Float32),
  DoubleVal(v: Float64),
  TextVal(v: Str),
}

fn f32_roundtrip() -> Int {
  var x = Value.FloatVal(2.5);
  match x {
    FloatVal(f) => { return float.float_bits(f as Float64); }
    _ => { return -1; }
  }
}

fn f64_roundtrip() -> Int {
  var x = Value.DoubleVal(2.5);
  match x {
    DoubleVal(d) => { return float.float_bits(d); }
    _ => { return -1; }
  }
}

fn int_roundtrip() -> Int {
  var x = Value.IntVal(44);
  match x {
    IntVal(n) => { return n; }
    _ => { return -1; }
  }
}

fn text_roundtrip_ok() -> Bool {
  var x = Value.TextVal("abc");
  match x {
    TextVal(s) => { return s == "abc"; }
    _ => { return false; }
  }
}

fn main() -> Int {
  if f32_roundtrip() != float.float_bits(2.5) { return 1; }
  if f64_roundtrip() != float.float_bits(2.5) { return 2; }
  if int_roundtrip() != 44 { return 3; }
  if !text_roundtrip_ok() { return 4; }
  return 0;
}
