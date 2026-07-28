// M35-T15: Derive[Eq] on struct — equality comparison
type Vec2 = { x: Int; y: Int; } derive[Eq]
type Triple = { a: Int; b: Int; c: Int; } derive[Eq]
fn main() -> Int {
  var v1: Vec2 = Vec2{ x: 1; y: 2; };
  var v2: Vec2 = Vec2{ x: 1; y: 2; };
  var v3: Vec2 = Vec2{ x: 3; y: 4; };
  if !(v1 == v2) { return 1; }
  if v1 == v3 { return 2; }
  if !(v1 != v3) { return 3; }
  var t1: Triple = Triple{ a: 1; b: 2; c: 3; };
  var t2: Triple = Triple{ a: 1; b: 2; c: 3; };
  var t3: Triple = Triple{ a: 1; b: 2; c: 4; };
  if !(t1 == t2) { return 4; }
  if t1 == t3 { return 5; }
  return 0;
}

