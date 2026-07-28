// M29: If-elif as expression
fn classify(score: Int) -> Int {
  return if score >= 90 { 4 } elif score >= 80 { 3 } elif score >= 70 { 2 } else { 1 };
}
fn main() -> Int {
  var r1 = classify(95);
  var r2 = classify(85);
  var r3 = classify(75);
  var r4 = classify(65);
  if r1 == 4 && r2 == 3 && r3 == 2 && r4 == 1 { return 0; }
  return 1;
}
