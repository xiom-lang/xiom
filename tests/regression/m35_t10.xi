// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T10: Struct with all field types -- Bool, Int, Int8/16/32/64, Float64, Str (no Char in large struct)
type MegaStruct = {
  f_bool: Bool;
  f_int: Int;
  f_int8: Int8;
  f_int16: Int16;
  f_int32: Int32;
  f_int64: Int64;
  f_float: Float64;
  f_str: Str;
}
fn mega_sum(s: MegaStruct) -> Int {
  var r: Int = 0;
  if s.f_bool { r = r + 1; }
  r = r + s.f_int + s.f_int8 as Int + s.f_int16 as Int + s.f_int32 as Int + s.f_int64 as Int;
  return r;
}
fn mega_check_str(s: MegaStruct) -> Bool { return s.f_str == "mega"; }
fn build_mega() -> MegaStruct {
  return MegaStruct{
    f_bool: true; f_int: 10; f_int8: 2 as Int8; f_int16: 3 as Int16;
    f_int32: 4 as Int32; f_int64: 5 as Int64; f_float: 3.14; f_str: "mega";
  };
}
fn if_field(s: MegaStruct) -> Int { if s.f_bool { return s.f_int; } return 0; }
fn main() -> Int {
  var s: MegaStruct = build_mega();
  if s.f_int != 10 { return 1; }
  if s.f_int8 != 2 as Int8 { return 2; }
  if s.f_int16 != 3 as Int16 { return 3; }
  if s.f_int32 != 4 as Int32 { return 4; }
  if s.f_int64 != 5 as Int64 { return 5; }
  if s.f_float != 3.14 { return 6; }
  if !mega_check_str(s) { return 7; }
  var sum: Int = mega_sum(s);
  var expected: Int = 1 + 10 + 2 + 3 + 4 + 5;
  if sum != expected { return 8; }
  if if_field(s) != 10 { return 9; }
  return 0;
}

