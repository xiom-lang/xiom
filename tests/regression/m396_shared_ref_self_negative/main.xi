// m396 lock: `&self` is immutable; only `&mut self` may store through the
// receiver.
type Point = { x: Int; }

fn Point.peek(&self) -> Int {
  self.x = 1;
  return self.x;
}

fn main() -> Int {
  return Point{ x: 0 }.peek();
}
