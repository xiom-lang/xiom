// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m38_round8_ref_payload -- round-8 (2026-08-20) regression:
// Option<&T> reference payloads -- rand.weighted_pick's Some(&items[i])
// payload read gave garbage on comparison (the slot ADDRESS was strcmp'd
// as the string; &T value uses now auto-deref via the "&T" xiom record).
// Also covers plain &Str params compared as Str values.
module m38_round8_ref_payload
use xiom.collections;
use xiom.rand;

fn eq_lit(s: &Str) -> Bool {
  return s == "hello";
}

fn pick_str(items: &Vec[Str]) -> Option<&Str> {
  if items.len() == 0 { return None; }
  return Some(&items[0]);
}

fn pick_int(items: &Vec[Int]) -> Option<&Int> {
  if items.len() == 0 { return None; }
  return Some(&items[0]);
}

fn main() -> Int {
  // 1. rand.weighted_pick Option<&Str> payload compared as a string (rw1).
  var items = Vec[Str].new();
  items.push("x");
  items.push("y");
  var weights = Vec[Float64].new();
  weights.push(1.0);
  weights.push(0.0);
  match rand.weighted_pick(&items, &weights) {
    Some(s) => { if s != "x" && s != "y" { return 1; } }
    None => { return 2; }
  }
  // 2. Option<&Int> payload with explicit deref.
  var w = Vec[Int].new();
  w.push(42);
  match pick_int(&w) {
    Some(r) => { if *r != 42 { return 3; } }
    None => { return 4; }
  }
  // 3. Option<&Str> payload from a user fn.
  match pick_str(&items) {
    Some(s) => { if s != "x" { return 5; } }
    None => { return 6; }
  }
  // 4. Plain &Str param compared as a Str value.
  var h = "hello";
  if !eq_lit(&h) { return 7; }
  return 0;
}
