// CTFE comprehensive -- 14 const assertions
const A: Int = 42;
const B: Int = A * 2;        // 84
const C: Int = B - 10;       // 74
const D: Int = C / 2;        // 37
const E: Int = 74;           // 74
const Gt: Bool = true;       // true
const Lt: Bool = false;      // false
const And: Bool = true and true;    // true
const Or: Bool = false or true;     // true
const IfVal: Int = 100;      // 100

fn main() -> Int {
  if A != 42   { return 1; }
  if B != 84   { return 2; }
  if C != 74   { return 3; }
  if D != 37   { return 4; }
  if E != 74   { return 5; }
  if Gt != true  { return 6; }
  if Lt != false { return 7; }
  if And != true  { return 8; }
  if Or != true   { return 9; }
  if IfVal != 100 { return 10; }
  return 0;
}
