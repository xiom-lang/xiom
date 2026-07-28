// M33-K01: Pipe closure — simple identity function
fn main() -> Int { var id = |x| x; if id(42) != 42 { return 1; } return 0; }
