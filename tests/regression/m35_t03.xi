// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T03: Float64 in every context -- var, param, return, struct, enum, array, if, while, operators
type FloatBox = { value: Float64; }
enum FloatOption { Some(v: Float64), None }
fn float_add(a: Float64, b: Float64) -> Float64 { return a + b; }
fn float_mul(a: Float64, b: Float64) -> Float64 { return a * b; }
fn float_sub(a: Float64, b: Float64) -> Float64 { return a - b; }
fn float_div(a: Float64, b: Float64) -> Float64 { return a / b; }
fn float_if(v: Float64) -> Int { if v > 0.0 { return 1; } return 0; }
fn float_loop_acc(n: Int) -> Float64 { var i: Int = 0; var acc: Float64 = 0.0; while i < n { acc = acc + 1.0; i = i + 1; } return acc; }
fn main() -> Int {
  var f1: Float64 = 10.0; var f2: Float64 = 4.0;
  var s: Float64 = float_add(f1, f2); if s != 14.0 { return 1; }
  var m: Float64 = float_mul(f1, 2.0); if m != 20.0 { return 2; }
  var d: Float64 = float_sub(f1, f2); if d != 6.0 { return 3; }
  var q: Float64 = float_div(f1, 2.0); if q != 5.0 { return 4; }
  if float_if(1.0) != 1 { return 5; }
  if float_if(-0.5) != 0 { return 6; }
  var box: FloatBox = FloatBox{ value: 9.0 }; if box.value != 9.0 { return 7; }
  var e: FloatOption = FloatOption.Some(7.0); match e { FloatOption.Some(v) => if v != 7.0 { return 8; } FloatOption.None => return 9; }
  var acc: Float64 = float_loop_acc(5); if acc != 5.0 { return 10; }
  var arr: Vec[Float64] = [1.0, 2.0, 3.0]; if arr[1] != 2.0 { return 11; }
  var neg: Float64 = -3.0; if neg != -3.0 { return 12; }
  var div_val: Float64 = 10.0 / 4.0; if div_val != 2.5 { return 13; }
  return 0;
}

