// M33-B16: Shadowing with ownership -- shadowed let creates new binding, original inaccessible
fn main() -> Int {
  let a = 1;
  let a = a + 10;
  let a = a * 3;
  if a == 33 { return 0; }
  return 1;
}
