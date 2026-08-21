// M33-K12: Closure in generic context -- generic apply function
fn apply[T](f: fn(T) -> T, x: T) -> T { return f(x); }
fn square(x: Int) -> Int { return x * x; }
fn main() -> Int { if apply(square, 5) != 25 { return 1; } return 0; }
