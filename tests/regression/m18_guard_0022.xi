module regression.m18_guard_0022

fn main() -> Int {
  var r: Result[Int, Str] = Err("not found");
  match r {
    Err(e) if e == "timeout" => { return 1; }
    Err(e) if e == "not found" => { return 0; }
    _ => { return 2; }
  }
}
