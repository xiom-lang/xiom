type D = { a: Int; b: Int; }
fn main() -> Int { var d = D{ a: 7, b: 8 }; if d.a+d.b != 15 { return 1; } return 0; }