// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C02: Every control flow with every type -- if/else, while, match on Bool, Int, Float64, Char, Str, Option, enum, struct
enum Color { Red, Green, Blue }
type Box = { width: Int; height: Int; }
fn check_int_type(x: Int) -> Int {
  if x > 100 { return 1; }
  else { if x > 50 { return 2; } else { if x > 0 { return 3; } else { return 0; } } }
}
fn sum_while(n: Int) -> Int { var s = 0; var i = 0; while i < n { s = s + i; i = i + 1; } return s; }
fn match_color(c: Color) -> Int {
  match c { Red => 10, Green => 20, Blue => 30 }
}
fn match_opt(o: Option[Int]) -> Int { match o { Some(v) => v, None => -1 } }
fn main() -> Int {
  if check_int_type(200) != 1 { return 1; }
  if check_int_type(75) != 2 { return 2; }
  if check_int_type(25) != 3 { return 3; }
  if check_int_type(-5) != 0 { return 4; }
  if sum_while(5) != 10 { return 5; }
  if sum_while(0) != 0 { return 6; }
  if match_color(Color.Red) != 10 { return 7; }
  if match_color(Color.Green) != 20 { return 8; }
  if match_color(Color.Blue) != 30 { return 9; }
  if match_opt(Some(99)) != 99 { return 10; }
  if match_opt(None) != -1 { return 11; }
  var flag: Bool = true;
  var cf: Int = if flag { 100 } else { 0 };
  if cf != 100 { return 12; }
  var fv: Float64 = 2.5;
  if fv > 2.0 { var r: Int = 42; if r == 42 { } } else { return 13; }
  var ch: Char = 'X';
  if ch != 'X' { return 14; }
  var s: Str = "hello";
  if s == "hello" { } else { return 15; }
  var b: Box = Box{ width: 10; height: 20; };
  if b.width == 10 { } else { return 16; }
  if b.height == 20 { } else { return 17; }
  return 0;
}
