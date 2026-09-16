// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z18: type_alias+const+struct+match+Option+Result+contract+compound_assign+module+while+generic
type Limit = Int;
const DEF_LIMIT: Limit = 50;
type View = { start: Int; end: Int; }
enum Cut { Full, Part(s: Int, e: Int), Single(i: Int) }
fn norm_window[T](v: View, s: Cut) -> Int
  requires: v.start >= 0
  requires: v.end >= v.start
  ensures: result >= 0
{
  match s {
    Full => v.end - v.start,
    Part(a, b) => { if a < v.start || b > v.end { return 0; } return b - a; }
    Single(i) => { if i < v.start || i > v.end { return 0; } return 1; }
  }
}
module win {
  pub fn norm(v: View, s: Cut) -> Int { return norm_window(v, s); }
  pub fn from_parts(s: Int, e: Int) -> View { return View{ start: s; end: e; }; }
  pub fn get_limit() -> Limit { return DEF_LIMIT; }
}
use win.norm;
use win.from_parts;
use win.get_limit;
fn main() -> Int {
  var w = from_parts(0, 100);
  var v1 = norm(w, Cut.Part(10, 30));
  var v2 = norm(w, Cut.Single(0));
  var v3 = norm(w, Cut.Part(200, 300));
  var chk = 0;
  if v1 == 20 { chk += 1; }
  if v2 == 1 { chk += 1; }
  if v3 == 0 { chk += 1; }
  if chk == 3 && get_limit() == 50 { return 0; }
  return 1;
}
