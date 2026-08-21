// M36-C06: Every derive with every struct shape -- Eq, Clone, Ord, Hash, Display, combinations
type Pt = { x: Int; y: Int; } derive[Eq]
type Rec = { val: Int; tag: Int; } derive[Clone]
type Meta = { id: Int; score: Int; } derive[Eq, Clone]
type Card = { suit: Int; rank: Int; } derive[Ord]
type Entry = { key: Int; data: Int; } derive[Eq, Ord]
type Label = { id: Int; text: Int; } derive[Display]
type Full = { a: Int; b: Int; c: Int; } derive[Eq, Clone, Ord]
type All = { i: Int; f: Float64; b: Bool; } derive[Eq, Clone, Display]
fn main() -> Int {
  var p1 = Pt{ x: 1; y: 2; };
  var p2 = Pt{ x: 1; y: 2; };
  var p3 = Pt{ x: 3; y: 4; };
  if !(p1 == p2) { return 1; }
  if p1 == p3 { return 2; }
  if !(p1 != p3) { return 3; }
  var r1 = Rec{ val: 10; tag: 20; };
  if r1.val != 10 { return 4; }
  var m1 = Meta{ id: 1; score: 100; };
  var m2 = Meta{ id: 1; score: 100; };
  if !(m1 == m2) { return 5; }
  var m3 = Meta{ id: 2; score: 100; };
  if m1 == m3 { return 6; }
  var c1 = Card{ suit: 1; rank: 5; };
  var c2 = Card{ suit: 1; rank: 5; };
  if c1.suit != c2.suit { return 7; }
  if c1.rank != c2.rank { return 8; }
  var e1 = Entry{ key: 1; data: 100; };
  var e2 = Entry{ key: 1; data: 100; };
  if !(e1 == e2) { return 9; }
  var e3 = Entry{ key: 2; data: 100; };
  if e1 == e3 { return 10; }
  var l1 = Label{ id: 1; text: 42; };
  if l1.text != 42 { return 11; }
  var f1 = Full{ a: 1; b: 2; c: 3; };
  var f2 = Full{ a: 1; b: 2; c: 3; };
  if !(f1 == f2) { return 12; }
  var a1 = All{ i: 1; f: 2.5; b: true; };
  if a1.i != 1 { return 13; }
  if a1.f < 2.49 || a1.f > 2.51 { return 14; }
  if !a1.b { return 15; }
  return 0;
}
