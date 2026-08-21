// E2E: DI emission -- compile with --debug and verify function names in binary
fn add(a: Int, b: Int) -> Int { return a + b; }
fn mul(a: Int, b: Int) -> Int { return a * b; }
fn main() -> Int { return add(mul(3, 4), 5); }
