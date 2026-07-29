module regression.m18_guard_0119

type Rect = { w: Int; h: Int; }

fn Rect.area(&self) -> Int { return w * h; }

fn main() -> Int {
  var r: Rect = { w: 10; h: 10; };
  match r {
    r if r.area() > 50 => { return 0; }
    _ => { return 1; }
  }
}
