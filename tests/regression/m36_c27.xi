// M36-C27: Every bool pattern — not, and, or, if, while, match, composite expressions, short-circuit eval
fn bool_not(b: Bool) -> Bool { return !b; }
fn bool_and(a: Bool, b: Bool) -> Bool { return a && b; }
fn bool_or(a: Bool, b: Bool) -> Bool { return a || b; }
fn bool_xor(a: Bool, b: Bool) -> Bool { return (a || b) && !(a && b); }
fn bool_eq(a: Bool, b: Bool) -> Bool { return a == b; }
fn bool_if(b: Bool) -> Int { if b { return 42; } return 0; }
fn bool_while(b: Bool, limit: Int) -> Int {
  var c = 0;
  var i = 0;
  while b && i < limit { c = c + 1; i = i + 1; }
  return c;
}
fn bool_match(b: Bool) -> Int { match b { true => 1, false => 0 } }
fn bool_short_and(a: Bool) -> Int {
  var x = 0;
  if a && true { x = 1; }
  return x;
}
fn bool_short_or(a: Bool) -> Int {
  var x = 0;
  if a || false { x = 1; }
  return x;
}
fn main() -> Int {
  if bool_not(true) != false { return 1; }
  if bool_not(false) != true { return 2; }
  if bool_not(bool_not(true)) != true { return 3; }
  if bool_and(true, true) != true { return 4; }
  if bool_and(true, false) != false { return 5; }
  if bool_and(false, true) != false { return 6; }
  if bool_and(false, false) != false { return 7; }
  if bool_or(true, true) != true { return 8; }
  if bool_or(true, false) != true { return 9; }
  if bool_or(false, true) != true { return 10; }
  if bool_or(false, false) != false { return 11; }
  if bool_xor(true, true) != false { return 12; }
  if bool_xor(true, false) != true { return 13; }
  if bool_xor(false, true) != true { return 14; }
  if bool_xor(false, false) != false { return 15; }
  if bool_eq(true, true) != true { return 16; }
  if bool_eq(true, false) != false { return 17; }
  if bool_if(true) != 42 { return 18; }
  if bool_if(false) != 0 { return 19; }
  if bool_while(true, 3) != 3 { return 20; }
  if bool_while(false, 100) != 0 { return 21; }
  if bool_match(true) != 1 { return 22; }
  if bool_match(false) != 0 { return 23; }
  if bool_short_and(true) != 1 { return 24; }
  if bool_short_and(false) != 0 { return 25; }
  if bool_short_or(true) != 1 { return 26; }
  if bool_short_or(false) != 0 { return 27; }
  var complex = (true && false) || (true && !false);
  if !complex { return 28; }
  var de_morgan = !(true && false) == (!true || !false);
  if !de_morgan { return 29; }
  return 0;
}
