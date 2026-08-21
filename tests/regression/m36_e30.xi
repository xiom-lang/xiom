// M36-E30: Very long expression -- all binary operators chained
fn main() -> Int {
  var result = 1 + 2 * 3 - 4 / 2 + 10 % 7 + 6 * 1 - 0 + 100 - 50 + 25 * 2 - 30 / 3 + 7 % 5 + 1 * 1 - 1 + 1;
  if result != 107 { return 1; }
  var bools = true && true || false && true || false || true && true;
  if !bools { return 2; }
  var cmp = 10 > 5 && 3 < 7 && 4 >= 2 && 8 <= 10 && 6 != 7 && 1 == 1;
  if !cmp { return 3; }
  return 0;
}