module m21_struct_mut_008
type Named = { name: Str; id: Int; }
fn main() -> Int {
  var n: Named = Named{ name: "alpha"; id: 0; };
  n.name = "beta";
  n.id = 99;
  if n.id == 99 { return 0; }
  return 1;
}
