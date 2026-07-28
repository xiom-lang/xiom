// M36-X29: Boolean logic chains — complex boolean expressions and short-circuit evaluation
fn main() -> Int {
  var a: Bool = true;
  var b: Bool = false;
  var c: Bool = true;
  var d: Bool = false;
  if (a && b) || (c && d) { return 1; }
  if !(a || b && c) { return 2; }
  if !a && !b != false { return 3; }
  var x: Int = 5;
  var y: Int = 10;
  var z: Int = 15;
  if (x > 0 && y < 20) != true { return 4; }
  if (x < 0 || y > 20) != false { return 5; }
  if !(x > 0) { return 6; }
  if (x < y && y < z) != true { return 7; }
  if (x > y || y > z) != false { return 8; }
  if !false && !false != true { return 9; }
  if !true || false != false { return 10; }
  var t1: Bool = (1 < 2) && (3 > 2);
  if !t1 { return 11; }
  var t2: Bool = (5 == 5) || (6 != 6);
  if !t2 { return 12; }
  var t3: Bool = !(10 > 20);
  if !t3 { return 13; }
  var complex: Bool = (x == 5 && y == 10) || (z == 15 && a);
  if !complex { return 14; }
  var chain: Bool = a && c && (x < y) && (y < z);
  if !chain { return 15; }
  return 0;
}
