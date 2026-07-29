module regression.m18_guard_0108

fn main() -> Int {
  var ch: Char = 'm';
  match ch {
    c if c > 'a' && c < 'z' => { return 0; }
    _ => { return 1; }
  }
}
