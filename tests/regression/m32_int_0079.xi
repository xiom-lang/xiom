// M32: Mixed-type multiplication Int16 * Int32 -> Int64
fn main() -> Int {
  var a: Int16 = 1000;
  var b: Int32 = 2000;
  var result: Int64 = a as Int64 * b as Int64;
  if result == 2000000 {
    return 0;
  }
  return 1;
}
