module m21_struct_mut_026
type Pair = { first: Int; second: Int; }

  fn swap(p: Pair) -> Pair {
    var tmp = p.first;
    var result: Pair = p;
    result.first = result.second;
    result.second = tmp;
    return result;
  }

pub fn run() -> Int {
    var p: Pair = { first: 10; second: 20; };
    var p2 = swap(p);
    if p2.first == 20 && p2.second == 10 { return 0; }
    return 1;
  }
use m21_struct_mut_026.run;
fn main() -> Int { return run(); }
