// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m301 lock (REL-20261010-1549-stdlib-4): CLAUSE-POSITION indexing of a
// Vec-typed `result` must resolve the element type exactly like a `var`
// binding does. The synthetic `result` local was registered in
// local_xiom_types but NOT in local_vec_elem, so `result[0]` in an ensures
// clause fell to the scalar emit_elem_load fallback, which loads the raw
// i64 slot:
//   * Vec[Float64]: the f64 BIT PATTERN was loaded as i64 and the
//     comparison sitofp'd it (1.0 -> 4.6e18) -> false
//     `contract violated: ensures` although the body had just stored 1.0;
//   * Vec[Vec[T]]: the loaded i64 was the row's inline first field (the
//     inner data pointer), inttoptr'd and read as a %struct.Vec header, so
//     `result[0].len()` read the inner ARRAY as the row. The known-failure
//     witness's Vec[Vec[Int]] control only "passed" because its row was
//     [1, 2] (the array's second element, read as the len field, happened
//     to equal the expected 2); the [7, 9] row below fails pre-fix.
// All shapes must exit 0 with runtime contract enforcement LIVE (m291);
// the Int [7, 9] control and the chained `result[0][0]` locks pin the
// coincidence and the chained-index paths.
module m301_clause_float_vec_index.main

fn f64_scalar_pair() -> Vec[Float64]
  ensures: result.len() == 2 && result[0] == 1.0 && result[1] == 2.0
{
  var out = Vec[Float64].new();
  out.push(1.0);
  out.push(2.0);
  return out;
}

fn f32_scalar_pair() -> Vec[Float32]
  ensures: result.len() == 2 && result[0] == 1.5 && result[1] == 2.5
{
  var out = Vec[Float32].new();
  out.push(1.5);
  out.push(2.5);
  return out;
}

fn f64_nested_rows() -> Vec[Vec[Float64]]
  ensures: result.len() == 2 && result[0].len() == 2 && result[1].len() == 2 && result[0][0] == 1.0 && result[0][1] == 2.0
{
  var out = Vec[Vec[Float64]].new();
  var r0 = Vec[Float64].new();
  r0.push(1.0);
  r0.push(2.0);
  out.push(r0);
  var r1 = Vec[Float64].new();
  r1.push(3.0);
  r1.push(4.0);
  out.push(r1);
  return out;
}

fn int_nested_nocoincidence() -> Vec[Vec[Int]]
  ensures: result.len() == 2 && result[0].len() == 2 && result[1].len() == 2 && result[0][0] == 7 && result[1][1] == 4
{
  var out = Vec[Vec[Int]].new();
  var r0 = Vec[Int].new();
  r0.push(7);
  r0.push(9);
  out.push(r0);
  var r1 = Vec[Int].new();
  r1.push(3);
  r1.push(4);
  out.push(r1);
  return out;
}

fn str_vec() -> Vec[Str]
  ensures: result.len() == 2 && result[0] == "aa" && result[1] == "bb"
{
  var out = Vec[Str].new();
  out.push("aa");
  out.push("bb");
  return out;
}

fn main() -> Int {
  var a = f64_scalar_pair();
  if a.len() != 2 { return 1; }
  if a[0] != 1.0 { return 2; }
  var b = f32_scalar_pair();
  if b.len() != 2 { return 3; }
  if b[1] != 2.5 { return 4; }
  var c = f64_nested_rows();
  if c[0][1] != 2.0 { return 5; }
  var d = int_nested_nocoincidence();
  if d[0].len() != 2 { return 6; }
  var e = str_vec();
  if e[0].len() != 2 { return 7; }
  return 0;
}
