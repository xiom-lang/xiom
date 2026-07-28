// M35-L05: Struct with Char fields — verify char layout and encoding
type CharPair = { first: Char; second: Char; third: Char; }

fn main() -> Int {
  var cp = CharPair{ first: 'A'; second: 'B'; third: 'C'; };
  if cp.first != 'A' { return 1; }
  if cp.second != 'B' { return 2; }
  if cp.third != 'C' { return 3; }
  var a: Int = 65;
  var b: Int = 66;
  var c: Int = 67;
  var ch1: Char = a as Char;
  var ch2: Char = b as Char;
  var ch3: Char = c as Char;
  if ch1 != 'A' { return 4; }
  if ch2 != 'B' { return 5; }
  if ch3 != 'C' { return 6; }
  return 0;
}
