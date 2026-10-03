// const-tables probe (relay report 2026-10-03): module-level const arrays
// with Str / struct payloads mis-materialize on v0.62.2 while [N]Int is
// correct. Deterministic. Expected: rc 0.
module const_tables_probe

use xiom.string;

type Row = { a: Int; b: Int; }

const K: [4]Int = [ 11, 22, 33, 44 ];
const NAMES: [3]Str = [ "alpha", "beta", "gamma" ];
const ROWS: [3]Row = [ Row{ a: 1; b: 2 }, Row{ a: 3; b: 4 }, Row{ a: 5; b: 6 } ];

fn main() -> Int {
  if K[0] != 11 { return 1; }
  if K[3] != 44 { return 2; }
  if NAMES[0] != "alpha" { return 3; }
  if NAMES[1] != "beta" { return 4; }
  if NAMES[2] != "gamma" { return 5; }
  if ROWS[0].a != 1 { return 6; }
  if ROWS[1].b != 4 { return 7; }
  if ROWS[2].a != 5 { return 8; }
  var total = 0;
  var i = 0;
  while i < 3 { total = total + string.str_len(NAMES[i]); i = i + 1; }
  if total != 14 { return 9; }
  return 0;
}
