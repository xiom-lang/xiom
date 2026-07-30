module m21_struct_mut_011
type Counter = { value: Int; }

  fn Counter.increment() -> Int {
    self.value = self.value + 1;
    return self.value;
  }

fn main() -> Int {
    var c: Counter = { value: 0; };
    c.increment();
    c.increment();
    if c.value == 2 { return 0; }
    return 1;
  }
use m21_struct_mut_011.run;
fn main() -> Int { return run(); }
