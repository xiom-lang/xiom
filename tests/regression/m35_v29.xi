// M35-V29: Vec iteration with while — scan forward and backward
use xiom.collections;

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(10);
  v.push(20);
  v.push(30);
  v.push(40);
  v.push(50);

  // Forward scan: sum
  var fsum = 0;
  var fi = 0;
  while fi < v.len() {
    fsum = fsum + v[fi];
    fi = fi + 1;
  }
  if fsum != 150 { return 1; }

  // Backward scan: build reversed
  var rev = Vec[Int].new();
  var bi = v.len() - 1;
  while bi >= 0 {
    rev.push(v[bi]);
    bi = bi - 1;
  }
  if rev.len() != 5 { return 2; }
  if rev[0] != 50 { return 3; }
  if rev[4] != 10 { return 4; }

  // Indexed access pattern
  if v.get(0) != Some(10) { return 5; }
  if v.get(2) != Some(30) { return 6; }
  if v.get(4) != Some(50) { return 7; }
  if v.get(5) != None { return 8; }

  return 0;
}
