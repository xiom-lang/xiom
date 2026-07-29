module regression.m18_guard_0104

fn get_limit() -> Result[Int, Str] { return Ok(100); }

fn main() -> Int {
  var x: Int = 50;
  match x {
    v if v > get_limit().unwrap_or(0) => { return 1; }
    v if v < get_limit().unwrap_or(0) => { return 0; }
    _ => { return 2; }
  }
}
