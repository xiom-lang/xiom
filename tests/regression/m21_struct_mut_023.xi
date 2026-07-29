module m21_struct_mut_023
enum Color { Red, Green, Blue }

  type ColoredPoint = { x: Int; y: Int; color: Color; }

  pub fn run() -> Int {
    var cp: ColoredPoint = { x: 0; y: 0; color: Color.Red; };
    cp.color = Color.Blue;
    match cp.color {
      Color.Blue => return 0,
      _ => return 1,
    }
  }
use m21_struct_mut_023.run;
fn main() -> Int { return run(); }
