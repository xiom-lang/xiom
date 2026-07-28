// M35-O21: Result map_err — match-based map over Err value
fn result_map_err(r: Result[Int, Str], f: fn(Str) -> Str) -> Result[Int, Str] {
  match r { Ok(v) => Ok(v), Err(e) => Err(f(e)) }
}
fn add_prefix(s: Str) -> Str { return "err: " + s; }
fn main() -> Int {
  match result_map_err(Ok(42), add_prefix) { Ok(v) => { if v != 42 { return 1; } } Err(_) => { return 2; } }
  match result_map_err(Err("fail"), add_prefix) { Ok(_) => { return 3; } Err(e) => { if e != "err: fail" { return 4; } } }
  return 0;
}
