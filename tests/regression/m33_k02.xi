// M33-K02: Pipe closure -- capturing local variable
fn main() -> Int { var base = 100; var f = |x| base + x; if f(23) != 123 { return 1; } return 0; }
