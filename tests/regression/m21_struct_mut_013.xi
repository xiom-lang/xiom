module m21_struct_mut_013
type Wrapper = Container{ data: Int; }

  fn Wrapper.set(v: Int) -> Wrapper {
    self.data = v;
    return self;
  }

  fn Wrapper.double() -> Wrapper {
    self.data = self.data * 2;
    return self;
  }

  pub fn run() -> Int {
    var w: Wrapper = Container{ data: 5; };
    w.set(10).double();
    if w.data == 20 { return 0; }
    return 1;
  }
use m21_struct_mut_013.run;
fn main() -> Int { return run(); }
