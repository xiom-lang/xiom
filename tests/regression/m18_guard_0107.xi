module regression.m18_guard_0107

fn main() -> Int {
  var ch: Char = 'x';
  match ch {
    c if c == 'x' => { return 0; }
    _ => { return 1; }
  }
}
