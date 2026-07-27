type B = { n: Int; }
fn B.double(self) -> Int { return self.n * 2; }
fn main() -> Int { var b = B{ n: 21 }; if b.double() != 42 { return 1; } return 0; }