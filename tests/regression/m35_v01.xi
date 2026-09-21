// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V01: Vec[Int] create, push, pop, len
fn main() -> Int {
  var v = Vec[Int].new();
  v.push(10);
  v.push(20);
  v.push(30);
  if v.len() != 3 { return 1; }
  var last = v.pop();
  if last != Some(30) { return 2; }
  if v.len() != 2 { return 3; }
  var mid = v.pop();
  if mid != Some(20) { return 4; }
  var first_pop = v.pop();
  if first_pop != Some(10) { return 5; }
  if v.len() != 0 { return 6; }
  var empty_pop = v.pop();
  if empty_pop != None { return 7; }
  return 0;
}
