module test {
  pub type Counter[T] = {
    val: T;
    count: Int;
  }
  pub fn Counter.new[T](initial: T) -> Counter[T] {
    return Counter[T]{ val: initial, count: 0 };
  }
  fn test() -> Int {
    var cb = Counter.new[Bool](false);
    return 1;
  }
  pub fn run() -> Int { return test(); }
}
use test.run;
fn main() -> Int { return run(); }
