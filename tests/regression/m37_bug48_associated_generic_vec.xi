// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// BUG 48-adjacent regression: generic fns taking &Vec[T] params. The caller
// passed the Vec BY VALUE while the mono def GEPs through a %struct.Vec*
// param -- the callee read the data pointer as the Vec header (AV in every
// contains-style generic fn). Also covers the associated-form interface
// dispatch (Eq[T].eq) inside generic fns.
module m37_bug48_associated_generic_vec
use xiom.collections;

interface Eq[T] {
  fn eq(a: T, b: T) -> Bool;
}

impl Eq[Int] {
  fn eq(a: Int, b: Int) -> Bool {
    return a == b;
  }
}

fn contains_eq[T: Eq](items: &Vec[T], value: T) -> Bool {
  var i = 0;
  while i < items.len() {
    if Eq[T].eq(items[i], value) {
      return true;
    }
    i = i + 1;
  }
  return false;
}

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(1);
  v.push(2);
  v.push(3);
  if !contains_eq(&v, 2) { return 1; }
  if contains_eq(&v, 9) { return 2; }
  return 0;
}
