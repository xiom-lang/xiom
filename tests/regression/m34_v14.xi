// M34-V14: Float in generic -- Option[Float64] pattern matching (arm correctness)
fn main() -> Int {
  var a = Some(1.0);
  var b = None;
  var ok: Int = 0;
  match a { Some(v) => { ok = ok + 1; } None => { return 1; } }
  match b { Some(v) => { return 2; } None => { ok = ok + 1; } }
  if ok == 2 { return 0; }
  return 3;
}
