// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z17: array+generic+while+compound_assign+enum+match+contract+module+derive+recursion
type Chunk = { offset: Int; count: Int; } derive[Eq]
enum Scan { Forward, Backward, Skip(n: Int) }
fn walk_chunk[T](c: Chunk, dir: Scan) -> Chunk
  requires: c.offset >= 0
  requires: c.count >= 0
  ensures: result.offset >= 0
  ensures: result.count >= 0
{
  match dir {
    Forward => { var r = c; r.offset += 1; return r; }
    Backward => { var r = c; if r.offset > 0 { r.offset -= 1; } return r; }
    Skip(n) => { var r = c; r.offset += n; return r; }
  }
}
fn array_sum(arr: Vec[Int], n: Int) -> Int {
  if n <= 0 { return 0; }
  return arr[n - 1] + array_sum(arr, n - 1);
}
module walker {
  pub fn step(c: Chunk, d: Scan) -> Chunk { return walk_chunk(c, d); }
  pub fn arr_total(arr: Vec[Int], n: Int) -> Int { return array_sum(arr, n); }
}
use walker.step;
use walker.arr_total;
fn main() -> Int {
  var c = Chunk{ offset: 5; count: 3; };
  var c1 = step(c, Scan.Forward);
  var c2 = step(c1, Scan.Skip(2));
  var c3 = step(c2, Scan.Backward);
  var vals = [1, 2, 3, 4, 5];
  var s = arr_total(vals, 5);
  var i = 0;
  var sum = 0;
  while i < 5 { sum += vals[i]; i += 1; }
  if c3.offset == 7 && c3.count == 3 && sum == 15 && s == sum { return 0; }
  return 1;
}
