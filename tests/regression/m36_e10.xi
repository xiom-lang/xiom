// M36-E10: Function with 20 parameters
fn sum20(p1: Int, p2: Int, p3: Int, p4: Int, p5: Int, p6: Int, p7: Int, p8: Int, p9: Int, p10: Int, p11: Int, p12: Int, p13: Int, p14: Int, p15: Int, p16: Int, p17: Int, p18: Int, p19: Int, p20: Int) -> Int {
  return p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 + p10 + p11 + p12 + p13 + p14 + p15 + p16 + p17 + p18 + p19 + p20;
}
fn main() -> Int {
  var r = sum20(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20);
  if r != 210 { return 1; }
  return 0;
}