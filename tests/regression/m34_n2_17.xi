// M34-N2-17: Deep match-with-if nesting — 4 levels of match+if interleaved
type N = enum { V(val: Int), }

fn check(v: N) -> Int {
  match v {
    N.V(x) => {
      if x > 10 {
        return x - 10;
      } else {
        if x > 5 {
          return x - 5;
        } else {
          return x;
        }
      }
    }
  }
}

fn main() -> Int {
  var a = check(N.V(25));
  var b = check(N.V(8));
  var c = check(N.V(3));
  if a == 15 && b == 3 && c == 3 { return 0; }
  return 1;
}
