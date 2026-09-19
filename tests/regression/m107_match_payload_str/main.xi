// R52 lock (playground L5-26/L5-36/L3-02): Str payloads flowing through
// matches and unwrap_or must keep the Str representation.
//  - Vec[Str].get(i) -> match Some(n) (was: pointer printed as an integer)
//  - match expr -> .to_str() (match slot must keep i8*)
//  - let p = Some("x"); p.unwrap_or("y") (unannotated Option local).
use xiom.io;

fn grade(s: Int) -> Str {
  if s >= 90 { return "A"; }
  return "F";
}

fn main() -> Int {
  let names: Vec[Str] = Vec[Str].new();
  names.push("Alice");
  names.push("Bob");

  let n0 = match names.get(0) { Some(n) => n, None => "?" };
  if n0 != "Alice" { return 1; }
  let n1 = match names.get(1) { Some(n) => n, None => "?" };
  if n1 != "Bob" { return 2; }

  let scores: Vec[Int] = Vec[Int].new();
  scores.push(95);
  let g = match scores.get(0) { Some(v) => grade(v), None => "?" };
  if g.to_str() != "A" { return 3; }

  let p = Some("DragonSlayer");
  let name = p.unwrap_or("Player1");
  if name.to_str() != "DragonSlayer" { return 4; }

  let q: Option[Str] = None;
  let fallback = q.unwrap_or("Player1");
  if fallback.to_str() != "Player1" { return 5; }

  io.println("ok");
  return 0;
}
