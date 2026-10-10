// m254 (stdlib tuple-literal element read): an inline Vec element read
// inside a tuple literal must keep the element's XIOM type. The tuple
// namers defaulted Index to "Int", so `v.push((pred[i], lab[i]))` built
// Tuple__Int__Int over a (Float64, Int) vector: the Float64 component was
// coerced to i64 and corrupted (binding the element first worked around it).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m254_tuple_literal_index_elem

fn build(pred: &Vec[Float64], lab: &Vec[Int]) -> Vec[(Float64, Int)] {
  var v = Vec[(Float64, Int)].new();
  var i = 0;
  while i < pred.len() {
    v.push((pred[i], lab[i]));
    i = i + 1;
  }
  return v;
}

fn main() -> Int {
  var pv = Vec[Float64].new();
  pv.push(0.1);
  pv.push(0.9);
  var lv = Vec[Int].new();
  lv.push(0);
  lv.push(1);
  let r = build(&pv, &lv);
  let e0 = r[0];
  let e1 = r[1];
  if !(e0.0 > 0.05 && e0.0 < 0.15) { return 1; }
  if !(e1.0 > 0.85 && e1.0 < 0.95) { return 2; }
  if e0.1 != 0 { return 3; }
  if e1.1 != 1 { return 4; }
  return 0;
}
