// M32: Integer in enum payload
enum Value { IntVal(x: Int), SmallVal(x: Int8), BigVal(x: Int64) }
fn extract(v: Value) -> Int {
  match v {
    Value.IntVal(x) => x,
    Value.SmallVal(x) => x as Int,
    Value.BigVal(x) => x as Int,
  }
}
fn main() -> Int {
  var a = Value.IntVal(42);
  var b = Value.SmallVal(7);
  var c = Value.BigVal(100);
  var sum: Int = extract(a) + extract(b) + extract(c);
  if sum == 149 {
    return 0;
  }
  return 1;
}
