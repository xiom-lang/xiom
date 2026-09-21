// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Test: enum with Bool variant payload + this-based methods
module tests.ecosystem.test_enum_bool

pub enum Flag {
  Off,
  On(val: Bool),
}

fn Flag.is_on() -> Bool {
  match this {
    On(_) => { return true; }
    _ => { return false; }
  }
}

fn Flag.get_on_val() -> Bool {
  match this {
    On(val) => { return val; }
    _ => { return false; }
  }
}

fn main() -> Int {
  var passed = 0; var total = 0;
  var f1 = Flag.On(true);
  total = total + 1; if Flag.is_on(&f1) { passed = passed + 1; }
  var f2 = Flag.On(false);
  total = total + 1; if Flag.is_on(&f2) { passed = passed + 1; }
  var f3 = Flag.On(true);
  total = total + 1; if Flag.get_on_val(&f3) { passed = passed + 1; }
  var f4 = Flag.On(false);
  total = total + 1; if !Flag.get_on_val(&f4) { passed = passed + 1; }
  var f5 = Flag.Off;
  total = total + 1; if !Flag.is_on(&f5) { passed = passed + 1; }
  if passed == total { return 0; }
  return 1;
}
