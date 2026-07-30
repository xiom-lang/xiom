module m21_struct_mut_023
type ColoredPoint = { x: Int; y: Int; color: Color; }
fn main() -> Int {
  var cp: ColoredPoint = ColoredPoint{ x: 0; y: 0; color: Color.Red; };
  cp.color = Color.Blue;
  match cp.color {
    Color.Blue => return 0,
    _ => return 1,
  }
  return 1;
}
