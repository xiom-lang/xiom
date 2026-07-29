module regression.m18_guard_0025

fn main() -> Int {
  var r: Result[Int, Str] = Ok(10);
  match r {
    Ok(v) if v >= 10 => { return 0; }
    Ok(v) => { return 1; }
    _ => { return 2; }
  }
}
