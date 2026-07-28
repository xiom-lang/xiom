// M35-D26: Bloom filter — multiple hash indexes pattern
fn main() -> Int {
  var b0: Bool = false;
  var b1: Bool = false;
  var b2: Bool = false;
  var b3: Bool = false;
  b0 = true;
  b1 = true;
  if b0 == false { return 1; }
  if b1 == false { return 2; }
  if b2 == true { return 3; }
  b2 = true;
  if b2 == false { return 4; }
  return 0;
}
