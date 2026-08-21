// M36-C15: Every enum variant count 1-10 -- enums with increasing variant count, tested via match
enum V1 { A }
enum V2 { A, B }
enum V3 { A, B, C }
enum V4 { A, B, C, D }
enum V5 { A, B, C, D, E }
enum V6 { A, B, C, D, E, F }
enum V7 { A, B, C, D, E, F, G }
enum V8 { A, B, C, D, E, F, G, H }
enum V9 { A, B, C, D, E, F, G, H, I }
enum V10 { A, B, C, D, E, F, G, H, I, J }
fn map_v1(v: V1) -> Int { match v { A => 1 } }
fn map_v2(v: V2) -> Int { match v { A => 1, B => 2 } }
fn map_v3(v: V3) -> Int { match v { A => 1, B => 2, C => 3 } }
fn map_v4(v: V4) -> Int { match v { A => 1, B => 2, C => 3, D => 4 } }
fn map_v5(v: V5) -> Int { match v { A => 1, B => 2, C => 3, D => 4, E => 5 } }
fn map_v6(v: V6) -> Int { match v { A => 1, B => 2, C => 3, D => 4, E => 5, F => 6 } }
fn map_v7(v: V7) -> Int { match v { A => 1, B => 2, C => 3, D => 4, E => 5, F => 6, G => 7 } }
fn map_v8(v: V8) -> Int { match v { A => 1, B => 2, C => 3, D => 4, E => 5, F => 6, G => 7, H => 8 } }
fn map_v9(v: V9) -> Int { match v { A => 1, B => 2, C => 3, D => 4, E => 5, F => 6, G => 7, H => 8, I => 9 } }
fn map_v10(v: V10) -> Int { match v { A => 1, B => 2, C => 3, D => 4, E => 5, F => 6, G => 7, H => 8, I => 9, J => 10 } }
fn main() -> Int {
  if map_v1(V1.A) != 1 { return 1; }
  if map_v2(V2.A) != 1 { return 2; }
  if map_v2(V2.B) != 2 { return 3; }
  if map_v3(V3.A) != 1 { return 4; }
  if map_v3(V3.B) != 2 { return 5; }
  if map_v3(V3.C) != 3 { return 6; }
  if map_v4(V4.D) != 4 { return 7; }
  if map_v5(V5.E) != 5 { return 8; }
  if map_v6(V6.F) != 6 { return 9; }
  if map_v7(V7.G) != 7 { return 10; }
  if map_v8(V8.H) != 8 { return 11; }
  if map_v9(V9.I) != 9 { return 12; }
  if map_v10(V10.A) != 1 { return 13; }
  if map_v10(V10.E) != 5 { return 14; }
  if map_v10(V10.J) != 10 { return 15; }
  return 0;
}
