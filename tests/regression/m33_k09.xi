// M33-K09: Function return type — call constructed closure inline
fn apply_and_run(n: Int, x: Int) -> Int { var f = fn(y: Int) -> Int { return y + n; }; return f(x); }
fn main() -> Int { if apply_and_run(5, 10) != 15 { return 1; } return 0; }
