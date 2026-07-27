type P = { x: Int; y: Int; }
fn origin() -> P { return P{ x: 0, y: 0 }; }
fn main() -> Int { var a = P{ x: 1, y: 2 }; var b = a; b.x = 99; if a.x != 1 { return 1; } if b.x != 99 { return 2; } if b.y != 2 { return 3; } var o = origin(); if o.x != 0 { return 4; } return 0; }