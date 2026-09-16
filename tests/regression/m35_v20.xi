// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V20: Vec[Int] clone pattern -- manual copy
use xiom.collections;

fn clone_vec(src: &Vec[Int]) -> Vec[Int] {
  var dst = Vec[Int].new();
  var i = 0;
  while i < src.len() {
    dst.push(src[i]);
    i = i + 1;
  }
  return dst;
}

fn main() -> Int {
  var orig = Vec[Int].new();
  orig.push(7);
  orig.push(14);
  orig.push(21);
  var copy = clone_vec(&orig);
  if copy.len() != orig.len() { return 1; }
  if copy[0] != 7 { return 2; }
  if copy[1] != 14 { return 3; }
  if copy[2] != 21 { return 4; }
  // Mutate original via remove+insert
  orig.remove(1);
  orig.insert(1, 999);
  if orig[1] != 999 { return 5; }
  if copy[1] != 14 { return 6; }
  // Mutate copy via remove+insert
  copy.remove(0);
  copy.insert(0, 888);
  if copy[0] != 888 { return 7; }
  if orig[0] != 7 { return 8; }
  return 0;
}
