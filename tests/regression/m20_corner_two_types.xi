type A = { x: Int; }
type B = { y: Int; }
fn A.to_b(self) -> Int { return self.x; }
fn B.from_a(self) -> Int { return self.y; }
fn main() -> Int { var a = A{ x: 5 }; var b = B{ y: 10 }; if a.to_b() != 5 { return 1; } if b.from_a() != 10 { return 2; } return 0; }