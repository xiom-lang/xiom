// M33-B07: Return a value (move) -- function takes ownership, returns owned value
fn take_and_return(x: Int) -> Int { return x * 2; }
fn main() -> Int {
  var a = 21;
  var b = take_and_return(a);
  if a == 21 && b == 42 { return 0; }
  return 1;
}
