module regression.m18_guard_0017

fn main() -> Int {
  var r: Result[Int, Str] = Ok(5);
  match r {
    Ok(v) if v > 10 => { return 1; }
    Ok(v) => { return 0; }
    _ => { return 2; }
  }
}
