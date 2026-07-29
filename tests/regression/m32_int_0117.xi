// M32: Int16 neg of min (-(-32768) wraps to -32768)
fn main() -> Int {
  var a: Int16 = -32768 as Int16;
  var b: Int16 = -a;
  if b == -32768 as Int16 { return 0; }
  return 1;
}
