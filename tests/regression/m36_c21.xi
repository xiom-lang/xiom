// M36-C21: Every derive combination -- Eq, Clone, Eq+Clone, Ord, Hash, Display, multi-combos on structs and enums
type A = { x: Int; } derive[Eq]
type B = { y: Int; } derive[Clone]
type C = { a: Int; b: Int; } derive[Eq, Clone]
type D = { v: Int; } derive[Ord]
type E = { k: Int; } derive[Hash]
type F = { lbl: Int; } derive[Display]
type G = { p: Int; q: Int; } derive[Eq, Clone, Ord]
type H = { id: Int; data: Int; } derive[Eq, Clone, Display]
type I = { suit: Int; rank: Int; } derive[Eq, Ord, Hash]
type J = { a: Int; b: Int; c: Int; } derive[Eq, Clone, Ord, Hash]
type K = { u: Int; v: Int; w: Int; } derive[Eq, Clone, Ord, Hash, Display]
enum S1 { One, Two } derive[Eq]
enum S2 { X, Y } derive[Clone]
enum S3 { P, Q } derive[Eq, Clone]
fn main() -> Int {
  var a1 = A{ x: 1; };
  var a2 = A{ x: 1; };
  if !(a1 == a2) { return 1; }
  var b1 = B{ y: 10; };
  var b2 = B{ y: 10; };
  if b1.y != b2.y { return 2; }
  var c1 = C{ a: 1; b: 2; };
  var c2 = C{ a: 1; b: 2; };
  if !(c1 == c2) { return 3; }
  var c3 = C{ a: 1; b: 3; };
  if c1 == c3 { return 4; }
  var d1 = D{ v: 5; };
  var d2 = D{ v: 5; };
  if d1.v != d2.v { return 5; }
  var e1 = E{ k: 1; };
  var e2 = E{ k: 1; };
  if e1.k != e2.k { return 6; }
  var f1 = F{ lbl: 42; };
  if f1.lbl != 42 { return 7; }
  var g1 = G{ p: 1; q: 2; };
  var g2 = G{ p: 1; q: 2; };
  if !(g1 == g2) { return 8; }
  var g3 = G{ p: 2; q: 1; };
  if g1 == g3 { return 9; }
  var h1 = H{ id: 1; data: 100; };
  var h2 = H{ id: 1; data: 100; };
  if !(h1 == h2) { return 10; }
  var i1 = I{ suit: 1; rank: 10; };
  var i2 = I{ suit: 1; rank: 10; };
  if !(i1 == i2) { return 11; }
  var j1 = J{ a: 1; b: 2; c: 3; };
  var j2 = J{ a: 1; b: 2; c: 3; };
  if !(j1 == j2) { return 12; }
  var k1 = K{ u: 1; v: 2; w: 3; };
  var k2 = K{ u: 1; v: 2; w: 3; };
  if !(k1 == k2) { return 13; }
  var s1a = S1.One;
  var s1b = S1.One;
  var s1c = S1.Two;
  if !(s1a == s1b) { return 14; }
  if s1a == s1c { return 15; }
  var s2a = S2.X;
  if s2a != S2.X { return 16; }
  var s3a = S3.P;
  var s3b = S3.P;
  if !(s3a == s3b) { return 17; }
  return 0;
}
