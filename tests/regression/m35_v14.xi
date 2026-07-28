// M35-V14: Vec of enum — Vec[Color] simple variant push/access
use xiom.collections;

type Color = enum { Red, Green, Blue, }

fn main() -> Int {
  var v = Vec[Color].new();
  v.push(Color.Red);
  v.push(Color.Green);
  v.push(Color.Blue);
  if v.len() != 3 { return 1; }
  if v[0] != Color.Red { return 2; }
  if v[1] != Color.Green { return 3; }
  if v[2] != Color.Blue { return 4; }
  // Remove last
  v.remove(2);
  if v.len() != 2 { return 5; }
  if v[0] != Color.Red { return 6; }
  if v[1] != Color.Green { return 7; }
  // Insert new element
  v.insert(1, Color.Blue);
  if v[0] != Color.Red { return 8; }
  if v[1] != Color.Blue { return 9; }
  if v[2] != Color.Green { return 10; }
  return 0;
}
