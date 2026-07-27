fn fac(n: Int) -> Int { if n <= 1 { return 1; } return n * fac(n - 1); }
fn main() -> Int { if fac(5) != 120 { return 1; } if fac(0) != 1 { return 2; } if fac(1) != 1 { return 3; } return 0; }