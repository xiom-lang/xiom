// M35-T16: Method on each type — impl for struct wrapper, enum
interface GetValue { fn get(self) -> Int; }
type IntHolder = { val: Int; }
impl GetValue for IntHolder { fn get(self) -> Int { return self.val; } }
type FloatHolder = { val: Float64; }
interface GetFloat { fn get(self) -> Float64; }
impl GetFloat for FloatHolder { fn get(self) -> Float64 { return self.val; } }
type BoolHolder = { val: Bool; }
interface GetBool { fn get(self) -> Bool; }
impl GetBool for BoolHolder { fn get(self) -> Bool { return self.val; } }
enum NumPair { TwoInts(a: Int, b: Int), TwoFloats(a: Float64, b: Float64) }
interface Sum { fn sum(self) -> Float64; }
impl Sum for NumPair {
  fn sum(self) -> Float64 {
    match self {
      NumPair.TwoInts(a, b) => (a + b) as Float64,
      NumPair.TwoFloats(a, b) => a + b,
    }
  }
}
fn main() -> Int {
  var ih = IntHolder{ val: 42 };
  if ih.get() != 42 { return 1; }
  var fh = FloatHolder{ val: 3.14 };
  if fh.get() != 3.14 { return 2; }
  var bh = BoolHolder{ val: true };
  if bh.get() == false { return 3; }
  var np1 = NumPair.TwoInts(10, 20);
  if np1.sum() != 30.0 { return 4; }
  var np2 = NumPair.TwoFloats(1.5, 2.5);
  if np2.sum() != 4.0 { return 5; }
  return 0;
}

