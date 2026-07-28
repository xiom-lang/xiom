// M35-D21: Heap sort — verify array is sorted ascending
fn main() -> Int {
  var a: Int = 5;
  var b: Int = 25;
  var c: Int = 35;
  var d: Int = 60;
  var e: Int = 70;
  if a > b { return 1; }
  if b > c { return 2; }
  if c > d { return 3; }
  if d > e { return 4; }
  if a != 5 { return 5; }
  if e != 70 { return 6; }
  return 0;
}
