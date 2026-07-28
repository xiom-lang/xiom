// M32-C10: Multi-field invariant — range-constrained rectangle
type Rectangle = { width: Int; height: Int; invariant: width > 0 && height > 0; }
fn area(r: Rectangle) -> Int {
  return r.width * r.height;
}
fn main() -> Int {
  var rect = Rectangle{ width: 10; height: 5; };
  var a: Int = area(rect);
  if a == 50 { return 0; }
  return 1;
}
