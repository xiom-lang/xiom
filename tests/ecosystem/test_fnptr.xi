module tests.ecosystem.test_fnptr

fn add_one() -> Int { return 1; }

fn main() -> Int {
  var v: Vec[fn() -> Int] = Vec[fn() -> Int].new();
  v.push(add_one);
  var f: fn() -> Int = v[0];
  return f();
}