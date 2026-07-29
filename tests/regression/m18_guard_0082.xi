module regression.m18_guard_0082

type Data = { x: Int; }

fn main() -> Int {
  var x: Int = 100;
  var d: Data = { x = 42; };
  match d {
    d if d.x > 0 => { return 0; }
    _ => { return 1; }
  }
}
