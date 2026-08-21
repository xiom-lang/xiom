// M33-A14: Array mutation -- mutate via struct wrapper with mutable fields
type Cell = { val: Int; }
fn main() -> Int {
  var a = Cell{ val: 1; };
  var b = Cell{ val: 2; };
  var c = Cell{ val: 3; };
  a.val = 99;
  b.val = 77;
  if a.val == 99 && b.val == 77 && c.val == 3 { return 0; }
  return 1;
}
