// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z22: closure+generic+enum+match+derive+module+contract+compound_assign+while+array
type Cell = { x: Int; y: Int; } derive[Eq]
enum Dir { Up, Down, Left, Right }
fn move_cell[T](c: Cell, d: Dir) -> Cell
  requires: c.x >= 0
  requires: c.y >= 0
  ensures: result.x >= 0
  ensures: result.y >= 0
{
  var r = c;
  match d { Up => { r.y += 1; } Down => { r.y -= 1; } Left => { r.x -= 1; } Right => { r.x += 1; } }
  if r.x < 0 { r.x = 0; }
  if r.y < 0 { r.y = 0; }
  return r;
}
fn path_len(path: Vec[Int], n: Int) -> Int {
  var i = 0;
  var s = 0;
  while i < n { s += path[i]; i += 1; }
  return s;
}
module grid {
  pub fn shift(c: Cell, d: Dir) -> Cell { return move_cell(c, d); }
  pub fn length(p: Vec[Int], n: Int) -> Int { return path_len(p, n); }
}
use grid.shift;
use grid.length;
fn main() -> Int {
  var c = Cell{ x: 3; y: 3; };
  var c1 = shift(c, Dir.Right);
  var c2 = shift(c, Dir.Up);
  var arr = [c1.x, c1.y, c2.x, c2.y];
  var total = length(arr, 4);
  var f = |x| x + total;
  var tr = f(0);
  if c1.x == 4 && c2.y == 4 && total == 14 && tr == 14 { return 0; }
  return 1;
}
