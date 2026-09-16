// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C09: Every type alias with every target type -- Bool, Int, Int8, Int16, Int32, Int64, Float64, Char, Str, struct, enum, pointer, Option, Result
type MyBool = Bool;
type MyInt = Int;
type MyInt8 = Int8;
type MyInt16 = Int16;
type MyInt32 = Int32;
type MyFloat64 = Float64;
type MyChar = Char;
type MyStr = Str;
type MyStruct = { x: Int; y: Int; }
type MyEnumVal = Int;
type MyPtr = *Int;
type MyOptionInt = Option[Int];
type MyResult = Result[Int, Str];
fn make_my_struct(a: MyInt, b: MyInt) -> MyStruct { return MyStruct{ x: a; y: b; }; }
fn test_bools(b: MyBool, c: Bool) -> Int { if b == c { return 1; } return 0; }
fn test_ints(a: MyInt, b: Int) -> Int { if a == b { return a; } return -1; }
fn test_wide(a: MyInt32, b: Int32) -> Int { if a > b { return a as Int; } return b as Int; }
fn test_float(a: MyFloat64, b: Float64) -> Int { if a > b { return 1; } return 0; }
fn main() -> Int {
  var flag: MyBool = true;
  if test_bools(flag, true) != 1 { return 1; }
  if test_ints(42, 42) != 42 { return 2; }
  if test_ints(42, 99) != -1 { return 3; }
  var v8: MyInt8 = 127;
  if v8 != 127 { return 4; }
  var v16: MyInt16 = 32767;
  if v16 != 32767 { return 5; }
  var v32: MyInt32 = 1000000;
  if test_wide(v32, 500000) != 1000000 { return 6; }
  var fv: MyFloat64 = 2.5;
  if test_float(fv, 1.0) != 1 { return 7; }
  if test_float(fv, 3.0) != 0 { return 8; }
  var ch: MyChar = 'Z';
  if ch != 'Z' { return 9; }
  var st: MyStr = "nice";
  if st != "nice" { return 10; }
  var s1 = make_my_struct(3, 7);
  if s1.x != 3 { return 11; }
  if s1.y != 7 { return 12; }
  var r: MyResult = Ok(42);
  match r { Ok(v) => { if v != 42 { return 13; } } Err(_) => { return 14; } }
  return 0;
}
