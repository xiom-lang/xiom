module regression.m18_guard_0120

type A = { x: Int; }

type B = { a: A; }

fn main() -> Int {
  var b: B = { a: { x: 7; }; };
  match b {
    b if b.a.x > 0 => { return 0; }
    _ => { return 1; }
  }
}
