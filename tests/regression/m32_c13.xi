// M32-C13: Struct invariant + contract combined
type Score = { points: Int; invariant: points >= 0; }
fn add_score(s: Score, delta: Int) -> Score
  requires: delta >= 0
  ensures: result.points == s.points + delta
{
  return Score{ points: s.points + delta; };
}
fn main() -> Int {
  var s = Score{ points: 50; };
  var s2 = add_score(s, 30);
  if s2.points == 80 { return 0; }
  return 1;
}
