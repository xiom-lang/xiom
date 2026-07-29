module regression.m18_guard_0090

type Num = { val: Int; }

fn Num.is_gt(&self, n: Int) -> Bool {
  return val > n;
}

fn main() -> Int {
  var v: Num = { val = 42; };
  match v {
    v if v.is_gt(10) => { return 0; }
    _ => { return 1; }
  }
}
