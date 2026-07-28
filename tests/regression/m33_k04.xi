// M33-K04: Block closure — typed params, direct assignment (no parens)
fn main() -> Int { var dbl = fn(x: Int) -> Int { return x * 2; }; if dbl(21) != 42 { return 1; } return 0; }
