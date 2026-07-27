enum Color { Red, Green, Blue }
fn is_red(c: Color) -> Bool { match c { Red => { return true; } _ => { return false; } } }
fn main() -> Int { if !is_red(Color.Red) { return 1; } if is_red(Color.Blue) { return 2; } return 0; }