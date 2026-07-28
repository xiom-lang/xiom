// M34-V05: Float addition precision (commutativity, associativity)
fn main() -> Int {
  var a: Float64 = 0.1;
  var b: Float64 = 0.2;
  var c: Float64 = 0.3;
  var sum1: Float64 = a + b;
  var sum2: Float64 = b + a;
  var sum3: Float64 = 1.5 + 2.5;
  if sum1 == sum2 && sum3 == 4.0 {
    return 0;
  }
  return 1;
}
