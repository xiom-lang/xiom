// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m220 (XVC-C-03): W004 must not fire on BARE unit-enum variants -- an
// all-unit-enum match with distinct bare arms is reachable in full. The
// lint treated every un-dotted Ident as a binding catch-all, so every arm
// after the first was flagged unreachable (18+ false positives in the
// XVECTOR suite).

module p_w004_bare;

enum Metric { Cosine, DotProduct, Euclidean }

fn label(m: Metric) -> Int {
  match m {
    Cosine => { return 1; },
    DotProduct => { return 2; },
    Euclidean => { return 3; },
  }
  return 0;
}

fn main() -> Int {
  if label(Metric.Cosine) != 1 { return 1; }
  if label(Metric.DotProduct) != 2 { return 2; }
  if label(Metric.Euclidean) != 3 { return 3; }
  return 0;
}
