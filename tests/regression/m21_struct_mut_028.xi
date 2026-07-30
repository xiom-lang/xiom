module m21_struct_mut_028
type Cell = { val: Int; }
fn main() -> Int {
  var v: Vec[Cell] = [{ val: 1; }, { val: 2; }, { val: 3; }];
  var v2 = update_at(v, 1, 99);
  if v2[0].val == 1 && v2[1].val == 99 && v2[2].val == 3 { return 0; }
  return 1;
  return 1;
}
