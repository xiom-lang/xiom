// M36-R26: very deep nesting -- if-else ladder stress test for parser recursion
fn main() -> Int {
  var x: Int = 0;
  if true { x = 1; } else { x = 2; }
  if x == 1 { x = 3; } else { x = 4; }
  if x == 3 { x = 5; } else { x = 6; }
  if x == 5 { x = 7; } else { x = 8; }
  if x == 7 { x = 9; } else { x = 10; }
  if x == 9 { x = 11; } else { x = 12; }
  if x == 11 { x = 13; } else { x = 14; }
  if x == 13 { x = 15; } else { x = 16; }
  if x == 15 { x = 17; } else { x = 18; }
  if x == 17 { x = 19; } else { x = 20; }
  if x == 19 { x = 21; } else { x = 22; }
  if x == 21 { x = 23; } else { x = 24; }
  if x == 23 { x = 25; } else { x = 26; }
  if x == 25 { x = 27; } else { x = 28; }
  if x == 27 { x = 29; } else { x = 30; }
  if x == 29 { x = 31; } else { x = 32; }
  if x == 31 { x = 33; } else { x = 34; }
  if x == 33 { x = 35; } else { x = 36; }
  if x == 35 { x = 37; } else { x = 38; }
  if x == 37 { x = 39; } else { x = 40; }
  if x == 39 { x = 41; } else { x = 42; }
  if x == 41 { x = 43; } else { x = 44; }
  if x == 43 { x = 45; } else { x = 46; }
  if x == 45 { x = 47; } else { x = 48; }
  if x == 47 { x = 49; } else { x = 50; }
  if x == 49 { x = 51; } else { x = 52; }
  if x == 51 { x = 53; } else { x = 54; }
  if x == 53 { x = 55; } else { x = 56; }
  if x == 55 { x = 57; } else { x = 58; }
  if x == 57 { x = 59; } else { x = 60; }
  if x == 59 { x = 61; } else { x = 62; }
  if x == 61 { x = 63; } else { x = 64; }
  if x == 63 { x = 65; } else { x = 66; }
  if x == 65 { x = 67; } else { x = 68; }
  if x == 67 { x = 69; } else { x = 70; }
  if x == 69 { x = 71; } else { x = 72; }
  if x == 71 { x = 73; } else { x = 74; }
  if x == 73 { x = 75; } else { x = 76; }
  if x == 75 { x = 77; } else { x = 78; }
  if x == 77 { x = 79; } else { x = 80; }
  if x == 79 { x = 81; } else { x = 82; }
  if x == 81 { x = 83; } else { x = 84; }
  if x == 83 { x = 85; } else { x = 86; }
  if x == 85 { x = 87; } else { x = 88; }
  if x == 87 { x = 89; } else { x = 90; }
  if x == 89 { x = 91; } else { x = 92; }
  if x == 91 { x = 93; } else { x = 94; }
  if x == 93 { x = 95; } else { x = 96; }
  if x == 95 { x = 97; } else { x = 98; }
  return 0;
}
