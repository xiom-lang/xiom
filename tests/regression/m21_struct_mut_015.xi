module m21_struct_mut_015
type Data = { val: Int; }
fn main() -> Int {
  var opt: Option[Data] = Some(Data{ val: 42; });
  var d = maybe_get(opt);
  d.val = 99;
  if d.val == 99 { return 0; }
  return 1;
  return 1;
}
