// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T07: Array in every context -- var, param, return, indexing, literal, multi-type arrays
fn sum_arr(arr: Vec[Int], n: Int) -> Int { var i: Int = 0; var s: Int = 0; while i < n { s = s + arr[i]; i = i + 1; } return s; }
fn first(arr: Vec[Int]) -> Int { return arr[0]; }
fn make_arr() -> Vec[Int] { var a: Vec[Int] = [10, 20, 30]; return a; }
fn array_param_check(arr: Vec[Bool]) -> Bool { return arr[0]; }
fn arr_if(arr: Vec[Int]) -> Int { if arr[0] > 0 { return arr[0]; } return 0; }
fn arr_while_sum(arr: Vec[Int], n: Int) -> Int { var i: Int = 0; var s: Int = 0; while i < n { s = s + arr[i]; i = i + 1; } return s; }
fn arr_match(arr: Vec[Int]) -> Int { if arr[0] == 1 { return 100; } if arr[0] == 2 { return 200; } return 0; }
type ArrBox = { items: Vec[Int]; }
fn arr_generic[T](arr: Vec[T]) -> T { return arr[0]; }
fn main() -> Int {
  var a1: Vec[Int] = [1, 2, 3, 4, 5];
  if sum_arr(a1, 5) != 15 { return 1; }
  if first(a1) != 1 { return 2; }
  var a2: Vec[Int] = make_arr();
  if a2[0] != 10 { return 3; }
  if a2[2] != 30 { return 4; }
  var ba: Vec[Bool] = [true, false];
  if !array_param_check(ba) { return 5; }
  if arr_if(a2) != 10 { return 6; }
  if arr_while_sum(a1, 3) != 6 { return 7; }
  if arr_match(a1) != 100 { return 8; }
  var items: Vec[Int] = [7, 8, 9];
  var box: ArrBox = ArrBox{ items: items };
  if box.items[1] != 8 { return 9; }
  var ga: Vec[Int] = [42];
  var g: Int = arr_generic(ga);
  if g != 42 { return 10; }
  var fa: Vec[Float64] = [1.5, 2.5, 3.5];
  if fa[1] != 2.5 { return 11; }
  var sa: Vec[Str] = ["x", "y", "z"];
  if sa[2] != "z" { return 12; }
  return 0;
}

