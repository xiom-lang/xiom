// M36-C10: Every cast direction between numeric types â€” Intâ†”Int8, Intâ†”Int16, Intâ†”Int32, Intâ†”Float64, Float64â†”Int, pointer casts
fn cast_i_to_i8(v: Int) -> Int8 { return v as Int8; }
fn cast_i8_to_i(v: Int8) -> Int { return v as Int; }
fn cast_i_to_i16(v: Int) -> Int16 { return v as Int16; }
fn cast_i16_to_i(v: Int16) -> Int { return v as Int; }
fn cast_i_to_i32(v: Int) -> Int32 { return v as Int32; }
fn cast_i32_to_i(v: Int32) -> Int { return v as Int; }
fn cast_i_to_f64(v: Int) -> Float64 { return v as Float64; }
fn cast_f64_to_i(v: Float64) -> Int { return v as Int; }
fn cast_ptr_to_int(p: *Int) -> Int
  requires: p != (0 as *Int)
{ unsafe { return p as Int; } }
fn cast_from_bool(b: Bool) -> Int { if b { return 1; } return 0; }
fn main() -> Int {
  if cast_i_to_i8(42) != 42 { return 1; }
  if cast_i8_to_i(42) != 42 { return 2; }
  if cast_i_to_i16(300) != 300 { return 3; }
  if cast_i16_to_i(300) != 300 { return 4; }
  if cast_i_to_i32(1000) != 1000 { return 5; }
  if cast_i32_to_i(1000) != 1000 { return 6; }
  var fv: Float64 = cast_i_to_f64(5);
  if fv < 4.99 || fv > 5.01 { return 7; }
  if cast_f64_to_i(7.9) != 7 { return 8; }
  if cast_f64_to_i(-3.2) != -3 { return 9; }
  if cast_f64_to_i(0.0) != 0 { return 10; }
  if cast_from_bool(true) != 1 { return 11; }
  if cast_from_bool(false) != 0 { return 12; }
  var i8v: Int8 = 100;
  var fi8v: Float64 = i8v as Float64;
  if fi8v < 99.9 || fi8v > 100.1 { return 13; }
  var i16v: Int16 = 200;
  var fi16v: Float64 = i16v as Float64;
  if fi16v < 199.9 || fi16v > 200.1 { return 14; }
  return 0;
}
