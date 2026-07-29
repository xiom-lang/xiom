module regression.m18_guard_0041

type Point = { x: Int; y: Int; }

fn main() -> Int {
  var p: Point = { x: 5; y: 10; };
  match p {
    p if p.x > 0 => { return 0; }
    _ => { return 1; }
  }
}
