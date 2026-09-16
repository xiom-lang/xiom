// fuzz seed: CTFE-friendly constant folding.
fn fold(n: Int) -> Int {
  var acc = 0;
  for i in 0..n {
    acc = acc + i * 2;
  }
  return acc;
}

fn main() -> Int {
  return fold(8) + 3;
}
