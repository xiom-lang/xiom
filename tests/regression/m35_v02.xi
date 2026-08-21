// M35-V02: Vec[Int] get/set index -- verify index-based read/write
use xiom.collections;

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(100);
  v.push(200);
  v.push(300);
  // Index read
  if v[0] != 100 { return 1; }
  if v[1] != 200 { return 2; }
  if v[2] != 300 { return 3; }
  // get via method
  var g0 = v.get(0);
  if g0 != Some(100) { return 4; }
  var g1 = v.get(1);
  if g1 != Some(200) { return 5; }
  var g_out = v.get(5);
  if g_out != None { return 6; }
  // Write via pop + push pattern (simulate set)
  v.remove(1);
  v.insert(1, 250);
  if v[1] != 250 { return 7; }
  if v[0] != 100 { return 8; }
  if v[2] != 300 { return 9; }
  return 0;
}
