// M35-L27: Alignment requirement — verify field alignment via struct literal access
type Aligned8 = { a: Int; b: Float64; }

fn check_fields(s: Aligned8) -> Int {
  if s.a != 100 { return 1; }
  if s.b > 199.9 && s.b < 200.1 {} else { return 2; }
  return 0;
}

fn main() -> Int {
  var s = Aligned8{ a: 100; b: 200.0; };
  if check_fields(s) != 0 { return 1; }
  return 0;
}
