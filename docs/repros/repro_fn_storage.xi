// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// REPRO R2 (BUG 27 #8): module-scope fn storage silently read-only.
// test/harness.xi: "g.f0 = f" on a module-level struct field no-ops;
// calling the stored fn calls the DEFAULT/null.
module repro_fn_storage

type FnBox = {
  f: fn(Int) -> Int;
}

var g = FnBox{ f: _id; };

fn _id(x: Int) -> Int {
  return x;
}

pub fn set_f(f: fn(Int) -> Int) {
  g.f = f;
}

pub fn call_f(x: Int) -> Int {
  return g.f(x);
}