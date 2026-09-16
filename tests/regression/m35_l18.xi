// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L18: Pointer to struct -- deref struct via function parameter pointer
type Vec2 = { x: Int; y: Int; }

fn get_x(p: *Vec2) -> Int {
  if p == unsafe { 0 as *Vec2 } { return -1; }
  var v: Int;
  unsafe { v = (*p).x; }
  return v;
}

fn get_y(p: *Vec2) -> Int {
  if p == unsafe { 0 as *Vec2 } { return -1; }
  var v: Int;
  unsafe { v = (*p).y; }
  return v;
}

fn main() -> Int {
  var nullp: *Vec2 = unsafe { 0 as *Vec2 };
  if get_x(nullp) != -1 { return 1; }
  if get_y(nullp) != -1 { return 2; }
  return 0;
}
