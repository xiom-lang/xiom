module regression.m18_guard_0023

fn main() -> Int {
  var r: Result[Int, Str] = Err("bad input");
  match r {
    Ok(v) if v > 0 => { return 1; }
    Err(e) if e == "bad input" => { return 0; }
    Err(e) => { return 2; }
    _ => { return 3; }
  }
}
