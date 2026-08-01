module m21_complex_generic_009
type Stack[T] = { items: Vec[T]; }

  pub fn Stack.new[T]() -> Stack[T] {
    return { items: []; };
  }

  fn Stack.push[T](val: T) {
    self.items.push(val);
  }

  fn Stack.top[T]() -> Option[T] {
    if self.items.len() > 0 {
      return Some(self.items[self.items.len() - 1]);
    }
    return None;
  }

  pub fn run() -> Int {
    var s: Stack[Int] = Stack.new();
    s.push(1);
    s.push(2);
    var top = s.top();
    match top {
      Some(v) => if v == 2 { return 0; } else { return 1; },
      None => return 1,
    }
  }
use m21_complex_generic_009.run;
fn main() -> Int { return run(); }
