// M33-P04: 50 local vars in one fn + all primitive types in 20-field struct + wide function table
// Pattern: fn main() -> Int { ... return 0; }

const CX0: Int = 0;
const CX1: Int = 1;
const CX2: Int = 2;
const CX3: Int = 3;
const CX4: Int = 4;
const CX5: Int = 5;
const CX6: Int = 6;
const CX7: Int = 7;
const CX8: Int = 8;
const CX9: Int = 9;

type PrimitiveStruct = {
  p00: Bool;
  p01: Char;
  p02: Int;
  p03: Float64;
  p04: Bool;
  p05: Char;
  p06: Int;
  p07: Float64;
  p08: Bool;
  p09: Char;
  p10: Int;
  p11: Float64;
  p12: Bool;
  p13: Char;
  p14: Int;
  p15: Float64;
  p16: Bool;
  p17: Char;
  p18: Int;
  p19: Float64;
}

fn init_primitive() -> PrimitiveStruct {
  return PrimitiveStruct{
    p00: true; p01: 'A'; p02: 1; p03: 1.0;
    p04: false; p05: 'B'; p06: 2; p07: 2.0;
    p08: true; p09: 'C'; p10: 3; p11: 3.0;
    p12: false; p13: 'D'; p14: 4; p15: 4.0;
    p16: true; p17: 'E'; p18: 5; p19: 5.0;
  };
}

fn sum_primitives(p: PrimitiveStruct) -> Int {
  var s: Int = 0;
  if p.p00 { s = s + 1; }
  if p.p04 { s = s + 1; }
  if p.p08 { s = s + 1; }
  if p.p12 { s = s + 1; }
  if p.p16 { s = s + 1; }
  s = s + p.p02 + p.p06 + p.p10 + p.p14 + p.p18;
  return s;
}

fn fifty_locals() -> Int {
  var v00: Int = 0; var v01: Int = 1; var v02: Int = 2; var v03: Int = 3; var v04: Int = 4;
  var v05: Int = 5; var v06: Int = 6; var v07: Int = 7; var v08: Int = 8; var v09: Int = 9;
  var v10: Int = 10; var v11: Int = 11; var v12: Int = 12; var v13: Int = 13; var v14: Int = 14;
  var v15: Int = 15; var v16: Int = 16; var v17: Int = 17; var v18: Int = 18; var v19: Int = 19;
  var v20: Int = 20; var v21: Int = 21; var v22: Int = 22; var v23: Int = 23; var v24: Int = 24;
  var v25: Int = 25; var v26: Int = 26; var v27: Int = 27; var v28: Int = 28; var v29: Int = 29;
  var v30: Int = 30; var v31: Int = 31; var v32: Int = 32; var v33: Int = 33; var v34: Int = 34;
  var v35: Int = 35; var v36: Int = 36; var v37: Int = 37; var v38: Int = 38; var v39: Int = 39;
  var v40: Int = 40; var v41: Int = 41; var v42: Int = 42; var v43: Int = 43; var v44: Int = 44;
  var v45: Int = 45; var v46: Int = 46; var v47: Int = 47; var v48: Int = 48; var v49: Int = 49;
  return v00+v01+v02+v03+v04+v05+v06+v07+v08+v09+v10+v11+v12+v13+v14+v15+v16+v17+v18+v19
    +v20+v21+v22+v23+v24+v25+v26+v27+v28+v29+v30+v31+v32+v33+v34+v35+v36+v37+v38+v39
    +v40+v41+v42+v43+v44+v45+v46+v47+v48+v49;
}

fn flat_a0() -> Int { return 1; }
fn flat_a1() -> Int { return 2; }
fn flat_a2() -> Int { return 3; }
fn flat_a3() -> Int { return 4; }
fn flat_a4() -> Int { return 5; }
fn flat_a5() -> Int { return 6; }
fn flat_a6() -> Int { return 7; }
fn flat_a7() -> Int { return 8; }
fn flat_a8() -> Int { return 9; }
fn flat_a9() -> Int { return 10; }
fn flat_b0() -> Int { return 11; }
fn flat_b1() -> Int { return 12; }
fn flat_b2() -> Int { return 13; }
fn flat_b3() -> Int { return 14; }
fn flat_b4() -> Int { return 15; }
fn flat_b5() -> Int { return 16; }
fn flat_b6() -> Int { return 17; }
fn flat_b7() -> Int { return 18; }
fn flat_b8() -> Int { return 19; }
fn flat_b9() -> Int { return 20; }
fn flat_c0() -> Int { return 21; }
fn flat_c1() -> Int { return 22; }
fn flat_c2() -> Int { return 23; }
fn flat_c3() -> Int { return 24; }
fn flat_c4() -> Int { return 25; }
fn flat_c5() -> Int { return 26; }
fn flat_c6() -> Int { return 27; }
fn flat_c7() -> Int { return 28; }
fn flat_c8() -> Int { return 29; }
fn flat_c9() -> Int { return 30; }
fn flat_d0() -> Int { return 31; }
fn flat_d1() -> Int { return 32; }
fn flat_d2() -> Int { return 33; }
fn flat_d3() -> Int { return 34; }
fn flat_d4() -> Int { return 35; }
fn flat_d5() -> Int { return 36; }
fn flat_d6() -> Int { return 37; }
fn flat_d7() -> Int { return 38; }
fn flat_d8() -> Int { return 39; }
fn flat_d9() -> Int { return 40; }
fn flat_e0() -> Int { return 41; }
fn flat_e1() -> Int { return 42; }
fn flat_e2() -> Int { return 43; }
fn flat_e3() -> Int { return 44; }
fn flat_e4() -> Int { return 45; }
fn flat_e5() -> Int { return 46; }
fn flat_e6() -> Int { return 47; }
fn flat_e7() -> Int { return 48; }
fn flat_e8() -> Int { return 49; }
fn flat_e9() -> Int { return 50; }

fn sum_flat() -> Int {
  return flat_a0()+flat_a1()+flat_a2()+flat_a3()+flat_a4()+flat_a5()+flat_a6()+flat_a7()+flat_a8()+flat_a9()
    +flat_b0()+flat_b1()+flat_b2()+flat_b3()+flat_b4()+flat_b5()+flat_b6()+flat_b7()+flat_b8()+flat_b9()
    +flat_c0()+flat_c1()+flat_c2()+flat_c3()+flat_c4()+flat_c5()+flat_c6()+flat_c7()+flat_c8()+flat_c9()
    +flat_d0()+flat_d1()+flat_d2()+flat_d3()+flat_d4()+flat_d5()+flat_d6()+flat_d7()+flat_d8()+flat_d9()
    +flat_e0()+flat_e1()+flat_e2()+flat_e3()+flat_e4()+flat_e5()+flat_e6()+flat_e7()+flat_e8()+flat_e9();
}

type Q00 = { a: Int; }
type Q01 = { a: Int; b: Int; }
type Q02 = { a: Int; b: Int; c: Int; }
type Q03 = { x: Float64; y: Float64; }
type Q04 = { flag: Bool; val: Int; }
type Q05 = { name: Str; count: Int; ok: Bool; }
type Q06 = { min: Int; max: Int; }
type Q07 = { source: Str; dest: Str; len: Int; }
type Q08 = { code: Int; msg: Str; }
type Q09 = { id: Int; parent: Int; depth: Int; }
type Q10 = { alpha: Float64; beta: Float64; gamma: Float64; }
type Q11 = { key: Char; value: Int; }
type Q12 = { start: Int; end: Int; }
type Q13 = { row: Int; col: Int; data: Float64; }
type Q14 = { head: Int; tail: Int; }
type Q15 = { version: Int; revision: Int; build: Int; }
type Q16 = { factor: Float64; offset: Float64; }
type Q17 = { total: Int; count: Int; avg: Float64; }
type Q18 = { r: Int; g: Int; b: Int; }
type Q19 = { from: Int; to: Int; weight: Float64; }

enum R00 { Zero, One, Two }
enum R01 { Small, Medium, Large }
enum R02 { Left, Right }
enum R03 { Up(v: Int), Down(v: Int) }
enum R04 { Good, Bad, Ugly(v: Int) }
enum R05 { Pass, Fail(code: Int) }
enum R06 { On, Off, Standby }
enum R07 { Hot, Cold, Warm(v: Int) }
enum R08 { Fast(v: Int), Slow }
enum R09 { Open, Closed, Locked(v: Str) }
enum R10 { Day, Night }
enum R11 { Ready, Running, Done }
enum R12 { WARNING(v: Str), ERROR(v: Str), INFO }
enum R13 { A, B, C, D, E }
enum R14 { Empty, Single(v: Int), Pair(v: Int, w: Int) }
enum R15 { Yes(v: Bool), No }

module prim_utils {
  pub fn validate_p(p: PrimitiveStruct) -> Bool {
    if p.p00 != true { return false; }
    if p.p01 != 'A' { return false; }
    if p.p02 != 1 { return false; }
    if p.p03 != 1.0 { return false; }
    return true;
  }
}

module stat_utils {
  pub fn sum(vals: Q17) -> Int { return vals.total; }
  pub fn avg_calc(total: Int, count: Int) -> Float64 {
    if count == 0 { return 0.0; }
    return total as Float64 / count as Float64;
  }
}

module compare_utils {
  pub fn eq_int(a: Int, b: Int) -> Bool { return a == b; }
  pub fn eq_float(a: Float64, b: Float64) -> Bool { return a == b; }
}

module nested_utils {
  use compare_utils.eq_int;
  use stat_utils.sum;
  pub fn verify_stat(q: Q17, expected: Int) -> Bool {
    return eq_int(sum(q), expected);
  }
}

module gate_utils {
  use nested_utils.verify_stat;
  pub fn check_all() -> Int {
    var q = Q17{ total: 100; count: 10; avg: 10.0; };
    if verify_stat(q, 100) == false { return 1; }
    return 0;
  }
}

fn struct_check() -> Int {
  var q00 = Q00{ a: 1; };
  var q02 = Q02{ a: 1; b: 2; c: 3; };
  var q05 = Q05{ name: "test"; count: 5; ok: true; };
  if q00.a != 1 { return 1; }
  if q02.a + q02.b + q02.c != 6 { return 2; }
  if q05.count != 5 { return 3; }
  return 0;
}

fn enum_check() -> Int {
  var r03 = R03.Up(42);
  match r03 {
    R03.Up(v) => if v == 42 { return 0; } else { return 1; },
    _ => return 2,
  }
}

fn interleaved_test() -> Int {
  var p = init_primitive();
  if p.p00 != true { return 1; }
  if p.p01 != 'A' { return 2; }
  var s = sum_primitives(p);
  if s != 18 { return 3; }
  return 0;
}

fn main() -> Int {
  var sum50 = fifty_locals();
  if sum50 != 1225 { return 1; }
  if sum_flat() != 1275 { return 2; }
  if struct_check() != 0 { return 3; }
  if enum_check() != 0 { return 4; }
  if interleaved_test() != 0 { return 5; }
  return 0;
}
