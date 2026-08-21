module m37_float_precision
// BUG 10 regression (docs/COMPILER_BUGS.md): float literals were emitted
// with {:.6} -- 6 decimals -- truncating 0.123456789 to 0.123457. The
// stored value must round-trip exactly (17 sig digits).

fn main() -> Int {
  var precise = 0.123456789;
  var truncated = 0.123457;
  if precise == truncated { return 1; }   // truncation would make them equal
  var pi = 3.14159265358979;
  if pi == 3.141593 { return 2; }         // truncation would make pi 3.141593
  if pi <= 3.1415926 { return 3; }
  var f = 1.5;
  if f != 1.5 { return 4; }
  return 0;
}
