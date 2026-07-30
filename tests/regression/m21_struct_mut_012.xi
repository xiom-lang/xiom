module m21_struct_mut_012
type Coords = { x: Int; y: Int; z: Int; }

  fn Coords.set_all(a: Int, b: Int, c: Int) {
    self.x = a;
    self.y = b;
    self.z = c;
  }

fn main() -> Int {
    var pt: Coords = { x: 0; y: 0; z: 0; };
    pt.set_all(3, 7, 11);
    if pt.x == 3 && pt.y == 7 && pt.z == 11 { return 0; }
    return 1;
  }
use m21_struct_mut_012.run;
fn main() -> Int { return run(); }
