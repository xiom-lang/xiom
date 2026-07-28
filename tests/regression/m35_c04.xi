// M35-C04: if elif elif elif else (5+ branches) — deep multi-way dispatch via nested else-if
fn classify(score: Int) -> Int {
  if score >= 90 { return 1; }
  else { if score >= 80 { return 2; }
  else { if score >= 70 { return 3; }
  else { if score >= 60 { return 4; }
  else { if score >= 50 { return 5; }
  else { return 6; } } } } }
}
fn main() -> Int {
  if classify(95) != 1 { return 1; }
  if classify(85) != 2 { return 2; }
  if classify(75) != 3 { return 3; }
  if classify(65) != 4 { return 4; }
  if classify(55) != 5 { return 5; }
  if classify(40) != 6 { return 6; }
  return 0;
}
