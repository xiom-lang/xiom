// M35-L20: Pointer to function -- function pointer pattern via indirect call
fn add(a: Int, b: Int) -> Int { return a + b; }
fn mul(a: Int, b: Int) -> Int { return a * b; }

fn apply_via_match(op: Int, x: Int, y: Int) -> Int {
  match op {
    1 => return add(x, y),
    2 => return mul(x, y),
    _ => return -1,
  }
}

fn main() -> Int {
  if apply_via_match(1, 3, 4) != 7 { return 1; }
  if apply_via_match(2, 3, 4) != 12 { return 2; }
  if apply_via_match(0, 0, 0) != -1 { return 3; }
  if add(10, 20) != 30 { return 4; }
  if mul(5, 6) != 30 { return 5; }
  return 0;
}
