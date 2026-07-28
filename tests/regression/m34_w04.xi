// M34-W04: Double NOT — ~(~x) == x on Int and UInt (avoid small-type ~ bug)
fn main() -> Int {
  var v_i: Int = 0x55AA;
  var v_u: UInt = 170;
  if ~(~(v_i)) == v_i && ~(~(v_u)) == v_u { return 0; }
  return 1;
}
