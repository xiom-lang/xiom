module m21_derive_007
interface Drawable {
    fn draw();
  }

  type Circle = { radius: Float64; }
  type Square = { side: Int; }

  pub fn run() -> Int {
    var c: Circle = { radius: 5.0; };
    if c.radius == 5.0 { return 0; }
    return 1;
  }
use m21_derive_007.run;
fn main() -> Int { return run(); }
