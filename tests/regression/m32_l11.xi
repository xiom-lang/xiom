// M32-L11: Loop counter -- count numbers divisible by 3 or 5 in 1..20
fn main() -> Int {
  var n: Int = 1;
  var count: Int = 0;
  while n <= 20 {
    if n % 3 == 0 || n % 5 == 0 { count += 1; }
    n += 1;
  }
  // 3,5,6,9,10,12,15,18,20 = 9 numbers
  if count == 9 { return 0; }
  return 1;
}
