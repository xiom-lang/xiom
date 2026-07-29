module m21_complex_generic_006
type Result[T, E] = enum { Ok(value: T), Err(error: E) }

  pub fn unwrap_or[T](r: Result[T, Int], default: T) -> T {
    match r {
      Result.Ok(v) => return v,
      Result.Err(_) => return default,
    }
  }

  pub fn run() -> Int {
    var r = Result.Ok(42);
    var v = unwrap_or[Int](r, 0);
    if v == 42 { return 0; }
    return 1;
  }
use m21_complex_generic_006.run;
fn main() -> Int { return run(); }
