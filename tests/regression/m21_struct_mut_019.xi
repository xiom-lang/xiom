module m21_struct_mut_019
type Rect = { x: Int; y: Int; w: Int; h: Int; }
fn main() -> Int {
  var r: Rect = Rect{ x: 10; y: 20; w: 100; h: 50; };
  var r2 = translate(r, 5, 3);
  if r2.x == 15 && r2.y == 23 && r2.w == 100 && r2.h == 50 { return 0; }
  return 1;
  return 1;
}
