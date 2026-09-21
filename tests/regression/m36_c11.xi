// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C11: Every array size -- literal arrays of sizes 1, 2, 5, 10, 50 using Vec-like patterns and indexing
fn sum_arr(arr: Vec[Int]) -> Int { var s = 0; var i = 0; while i < arr.len() { s = s + arr[i]; i = i + 1; } return s; }
fn find_arr(arr: Vec[Int], target: Int) -> Bool { var i = 0; while i < arr.len() { if arr[i] == target { return true; } i = i + 1; } return false; }
fn main() -> Int {
  var a1 = Vec[Int].new();
  a1.push(42);
  if a1.len() != 1 { return 1; }
  if a1[0] != 42 { return 2; }
  var a2 = Vec[Int].new();
  a2.push(10); a2.push(20);
  if a2.len() != 2 { return 3; }
  if a2[0] != 10 { return 4; }
  if a2[1] != 20 { return 5; }
  if sum_arr(a2) != 30 { return 6; }
  var a5 = Vec[Int].new();
  a5.push(1); a5.push(2); a5.push(3); a5.push(4); a5.push(5);
  if a5.len() != 5 { return 7; }
  var s5 = sum_arr(a5);
  if s5 != 15 { return 8; }
  var a10 = Vec[Int].new();
  var i10 = 0;
  while i10 < 10 { a10.push(i10 * 2); i10 = i10 + 1; }
  if a10.len() != 10 { return 9; }
  if a10[0] != 0 { return 10; }
  if a10[9] != 18 { return 11; }
  if sum_arr(a10) != 90 { return 12; }
  var a50 = Vec[Int].new();
  var i50 = 0;
  while i50 < 50 { a50.push(i50); i50 = i50 + 1; }
  if a50.len() != 50 { return 13; }
  if a50[0] != 0 { return 14; }
  if a50[49] != 49 { return 15; }
  var total50 = sum_arr(a50);
  if total50 != 1225 { return 16; }
  if find_arr(a5, 3) != true { return 17; }
  if find_arr(a5, 99) != false { return 18; }
  if find_arr(a10, 18) != true { return 19; }
  if find_arr(a10, 19) != false { return 20; }
  return 0;
}
