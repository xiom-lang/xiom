// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-A30: Rotate array -- verify element positions after left rotation
fn main() -> Int {
  var n: Int = 7;
  var k: Int = 2;
  var i: Int = 0;
  var ok: Int = 1;
  while i < n {
    var expected: Int = i + k + 1;
    if expected > n { expected = expected - n; }
    var idx: Int = i + k;
    if idx >= n { idx = idx - n; }
    if idx + 1 != expected { ok = 0; }
    i = i + 1;
  }
  if ok != 1 { return 1; }
  var n2: Int = 5;
  var k2: Int = 3;
  var i2: Int = 0;
  while i2 < n2 {
    var idx2: Int = (i2 + k2) % n2;
    if idx2 != ((i2 + k2) - ((i2 + k2) / n2) * n2) { return 2; }
    i2 = i2 + 1;
  }
  return 0;
}
