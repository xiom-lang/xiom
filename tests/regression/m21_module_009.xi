module m21_module_009
pub fn identity[T](x: T) -> T { return x; }

  pub fn run() -> Int {
    var a = identity[Int](42);
    if a == 42 { return 0; }
    return 1;
  }
use m21_module_009.run;
fn main() -> Int { return run(); }
