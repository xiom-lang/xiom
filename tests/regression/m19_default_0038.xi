// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0038

interface Readiness {
  fn ready(&self) -> Bool { return count() >= threshold(); }
  fn count(&self) -> Int;
  fn threshold(&self) -> Int;
}

type Inventory = { cnt: Int; min: Int; }

fn Inventory.ready(self) -> Bool { return self.count() >= self.threshold(); }


fn Inventory.count(&self) -> Int { return cnt; }

fn Inventory.threshold(&self) -> Int { return min; }

fn main() -> Int {
  var ok: Inventory = Inventory{ cnt: 10, min: 5 };
  var low: Inventory = Inventory{ cnt: 3, min: 5 };
  var eq: Inventory = Inventory{ cnt: 5, min: 5 };
  if ok.ready() && !low.ready() && eq.ready() { return 0; }
  return 1;
}
