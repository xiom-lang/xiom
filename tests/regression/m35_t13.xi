// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T13: Type alias for primitives -- no arithmetic on alias, only assignment
type MyBool = Bool;
type MyInt = Int;
type MyInt8 = Int8;
type MyInt16 = Int16;
type MyInt32 = Int32;
type MyInt64 = Int64;
type MyFloat = Float64;
type MyChar = Char;
type MyStr = Str;
fn alias_check() -> Int {
  var b: MyBool = true;
  if b == false { return 1; }
  var i: MyInt = 42;
  if i != 42 as MyInt { return 2; }
  var i8: MyInt8 = 7 as MyInt8;
  if i8 != 7 as MyInt8 { return 3; }
  var i16: MyInt16 = 77 as MyInt16;
  if i16 != 77 as MyInt16 { return 4; }
  var i32: MyInt32 = 777 as MyInt32;
  if i32 != 777 as MyInt32 { return 5; }
  var i64: MyInt64 = 7777 as MyInt64;
  if i64 != 7777 as MyInt64 { return 6; }
  var f: MyFloat = 2.0;
  if f != 2.0 { return 7; }
  var c: MyChar = 'Z';
  if c != 'Z' { return 8; }
  var s: MyStr = "alias";
  if s != "alias" { return 9; }
  return 0;
}
fn main() -> Int {
  var r1: Int = alias_check();
  if r1 != 0 { return r1; }
  return 0;
}

