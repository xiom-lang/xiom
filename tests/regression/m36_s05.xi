// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S05: Type checker -- type comparison and equality
type TypeInfo = { tid: Int; size: Int; align: Int; }
fn type_eq(a: TypeInfo, b: TypeInfo) -> Bool {
  return a.tid == b.tid && a.size == b.size && a.align == b.align;
}
fn type_is_int(t: TypeInfo) -> Bool {
  return t.tid == 1 && t.size >= 1 && t.size <= 8;
}
fn type_is_float(t: TypeInfo) -> Bool {
  return t.tid == 2 && (t.size == 4 || t.size == 8);
}
fn type_is_bool_or_char(t: TypeInfo) -> Bool {
  return t.tid == 3 || t.tid == 4;
}
fn type_ne(a: TypeInfo, b: TypeInfo) -> Bool {
  return !type_eq(a, b);
}
fn main() -> Int {
  var int32 = TypeInfo{ tid: 1; size: 4; align: 4; };
  var int64 = TypeInfo{ tid: 1; size: 8; align: 8; };
  var float64 = TypeInfo{ tid: 2; size: 8; align: 8; };
  var bool_t = TypeInfo{ tid: 3; size: 1; align: 1; };
  var char_t = TypeInfo{ tid: 4; size: 1; align: 1; };
  if !type_is_int(int32) { return 1; }
  if !type_is_int(int64) { return 2; }
  if type_is_int(float64) { return 3; }
  if !type_is_float(float64) { return 4; }
  if type_is_float(int32) { return 5; }
  if type_is_bool_or_char(int32) { return 6; }
  if !type_is_bool_or_char(bool_t) { return 7; }
  if !type_is_bool_or_char(char_t) { return 8; }
  if !type_ne(int32, float64) { return 9; }
  if type_ne(int32, int32) { return 10; }
  if !type_eq(int64, int64) { return 11; }
  return 0;
}
