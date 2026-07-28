// M34-H18: Cast in while condition — loop guard with cast expression
fn main() -> Int {
  var x: Int64 = 5;
  var count: Int = 0;
  var limit: Int8 = 5;
  while (x as Int) > 0 {
    x = x - 1;
    count = count + 1;
  }
  if count == limit as Int { return 0; }
  return 1;
}
