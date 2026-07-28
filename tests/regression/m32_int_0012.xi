// M32: Int16 maximum value 32767
fn main() -> Int {
  var x: Int16 = 32767;
  var y: Int16 = 32766;
  if x > y && x == 32767 as Int16 {
    return 0;
  }
  return 1;
}
