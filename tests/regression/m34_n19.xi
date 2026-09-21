// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N19: Type with Eq+Ord+Hash combine + Clone+Eq+Display combine
type Card = { suit: Int; rank: Int; } derive[Eq, Ord, Hash]
type Widget = { id: Int; label: Str; } derive[Clone, Eq, Display]
fn main() -> Int {
  var c1 = Card{ suit: 1; rank: 10; };
  var c2 = Card{ suit: 1; rank: 10; };
  var c3 = Card{ suit: 2; rank: 5; };
  var w1 = Widget{ id: 1; label: "btn"; };
  var w2 = w1.clone();
  if c1 == c2 && c1 != c3 && c1.rank > c3.rank && w1 == w2 { return 0; }
  return 1;
}
