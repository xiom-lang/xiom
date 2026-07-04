module shapes {
  pub type Rect = {
    w: Int;
    h: Int;
  }

  pub fn Rect.area() -> Int {
    return w * h;
  }

  pub fn Rect.new(w: Int, h: Int) -> Rect {
    return Rect{ w: w, h: h };
  }

  fn test() -> Int {
    var r = Rect.new(10, 20);
    if r.area() == 200 { return 1; }
    return 0;
  }

  pub fn run_test() -> Int { return test(); }
}

use shapes.run_test;

fn main() -> Int { return run_test(); }
