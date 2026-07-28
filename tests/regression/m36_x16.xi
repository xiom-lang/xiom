// M36-X16: Interface dispatch — multiple interfaces with multiple implementations
interface Measurable {
  fn measure(self) -> Int;
}
interface Describable {
  fn describe(self) -> Str;
}
type Box = { width: Int; height: Int; depth: Int; label: Str; }
impl Measurable for Box {
  fn measure(self) -> Int {
    return self.width * self.height * self.depth;
  }
}
impl Describable for Box {
  fn describe(self) -> Str { return self.label; }
}
type Circle = { radius: Int; name: Str; }
impl Measurable for Circle {
  fn measure(self) -> Int {
    return 3 * self.radius * self.radius;
  }
}
impl Describable for Circle {
  fn describe(self) -> Str { return self.name; }
}
fn total_volume(a: Box, b: Box) -> Int {
  return a.measure() + b.measure();
}
fn main() -> Int {
  var b1 = Box{ width: 2; height: 3; depth: 4; label: "box1"; };
  var b2 = Box{ width: 1; height: 2; depth: 3; label: "box2"; };
  var c = Circle{ radius: 5; name: "circle"; };
  if b1.measure() != 24 { return 1; }
  if b2.measure() != 6 { return 2; }
  if c.measure() != 75 { return 3; }
  if total_volume(b1, b2) != 30 { return 4; }
  if b1.describe() != "box1" { return 5; }
  if c.describe() != "circle" { return 6; }
  return 0;
}
