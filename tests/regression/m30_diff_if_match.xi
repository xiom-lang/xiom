// M30: Differential — if-chain vs match must produce equivalent results
fn via_if(x: Int) -> Int {
  if x == 1 { return 10; }
  elif x == 2 { return 20; }
  elif x == 3 { return 30; }
  return 0;
}
fn via_match(x: Int) -> Int {
  match x { 1 => 10, 2 => 20, 3 => 30, _ => 0, }
}
fn main() -> Int {
  if via_if(1) == via_match(1) && via_if(2) == via_match(2) && via_if(3) == via_match(3) { return 0; }
  return 1;
}
