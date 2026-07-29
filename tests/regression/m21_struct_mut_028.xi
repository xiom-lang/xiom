module m21_struct_mut_028
type Cell = { val: Int; }

  fn update_at(vec: Vec[Cell], idx: Int, new_val: Int) -> Vec[Cell] {
    var result = vec;
    result[idx].val = new_val;
    return result;
  }

  pub fn run() -> Int {
    var v: Vec[Cell] = [{ val: 1; }, { val: 2; }, { val: 3; }];
    var v2 = update_at(v, 1, 99);
    if v2[0].val == 1 && v2[1].val == 99 && v2[2].val == 3 { return 0; }
    return 1;
  }
use m21_struct_mut_028.run;
fn main() -> Int { return run(); }
