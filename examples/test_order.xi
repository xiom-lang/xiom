// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module a {
  pub type Data = {
    value: Int;
  }
  pub fn Data.new(v: Int) -> Data {
    return Data{ value: v };
  }
  pub fn Data.set(new_val: Int) -> Data {
    return Data{ value: new_val };
  }
  fn test_a() -> Int {
    var d = Data.new(42);
    return d.value;
  }
  pub fn run_a() -> Int { return test_a(); }
}

module b {
  pub type Stuff = {
    x: Int;
  }
  pub fn Stuff.new(x: Int) -> Stuff {
    return Stuff{ x: x };
  }
  fn test_b() -> Int {
    var s = Stuff.new(99);
    return s.x;
  }
  pub fn run_b() -> Int { return test_b(); }
}

module c {
  pub type Thing = {
    y: Int;
  }
  pub fn Thing.new(y: Int) -> Thing {
    return Thing{ y: y };
  }
  pub fn Thing.set(new_val: Int) -> Thing {
    return Thing{ y: new_val };
  }
  fn test_c() -> Int {
    var t = Thing.new(0);
    var t2 = t.set(100);
    return t2.y;
  }
  pub fn run_c() -> Int { return test_c(); }
}

use a.run_a;
use b.run_b;
use c.run_c;

fn main() -> Int {
  var total = 0;
  total = total + run_a();
  total = total + run_b();
  total = total + run_c();
  return total;
}
