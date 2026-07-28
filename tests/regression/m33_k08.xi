// M33-K08: Function as parameter type — call named fn via higher-order
fn call_twice(f: fn(Int) -> Int, x: Int) -> Int { var a = f(x); return f(a); }
fn inc(x: Int) -> Int { return x + 1; }
fn main() -> Int { if call_twice(inc, 10) != 12 { return 1; } return 0; }
