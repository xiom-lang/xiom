module regression.m18_guard_0043

type Point = { x: Int; y: Int; }

fn main() -> Int {
  var p: Point = { x: 3; y: 7; };
  match p {
    p if p.x > 0 && p.y > 0 => { return 0; }
    _ => { return 1; }
  }
}
