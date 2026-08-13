// M33-P13: 30-fn deep chain + 20 struct + 15 enum + 10 const + 5 modules all interleaved + big match + 50 vars
// Pattern: fn main() -> Int { ... return 0; }

const P13_KK0: Int = 0; const P13_KK1: Int = 1; const P13_KK2: Int = 2; const P13_KK3: Int = 3; const P13_KK4: Int = 4;
const P13_KK5: Int = 5; const P13_KK6: Int = 6; const P13_KK7: Int = 7; const P13_KK8: Int = 8; const P13_KK9: Int = 9;

fn z00() -> Int { return P13_KK0; }
type Z00_S = { id: Int; }
fn z01() -> Int { return z00() + 1; }
enum Z00_E { X0, X1 }
fn z02() -> Int { return z01() + 2; }
type Z01_S = { x: Float64; y: Float64; }
fn z03() -> Int { return z02() + 3; }
enum Z01_E { Y0(v: Int), Y1 }
fn z04() -> Int { return z03() + 4; }
type Z02_S = { label: Str; count: Int; ok: Bool; }
fn z05() -> Int { return z04() + 5; }
enum Z02_E { W0, W1, W2, W3 }
fn z06() -> Int { return z05() + 6; }
type Z03_S = { min: Int; max: Int; avg: Float64; }
fn z07() -> Int { return z06() + 7; }
enum Z03_E { V0(v: Str), V1(v: Int) }
fn z08() -> Int { return z07() + 8; }
type Z04_S = { key: Char; val: Int; }
fn z09() -> Int { return z08() + 9; }
enum Z04_E { U0, U1(v: Float64) }
fn z10() -> Int { return z09() + 10; }
type Z05_S = { r: Int; g: Int; b: Int; }
fn z11() -> Int { return z10() + 11; }
enum Z05_E { T0, T1, T2, T3, T4(v: Int) }
fn z12() -> Int { return z11() + 12; }
type Z06_S = { width: Int; height: Int; }
fn z13() -> Int { return z12() + 13; }
enum Z06_E { S0, S1(v: Int, w: Int) }
fn z14() -> Int { return z13() + 14; }
type Z07_S = { head: Int; tail: Int; size: Int; }
fn z15() -> Int { return z14() + 15; }
enum Z07_E { R0, R1, R2 }
fn z16() -> Int { return z15() + 16; }
type Z08_S = { code: Int; msg: Str; }
fn z17() -> Int { return z16() + 17; }
enum Z08_E { Q0(v: Bool), Q1 }
fn z18() -> Int { return z17() + 18; }
type Z09_S = { total: Int; items: Int; rate: Float64; }
fn z19() -> Int { return z18() + 19; }
enum Z09_E { P0, P1, P2, P3, P4 }
fn z20() -> Int { return z19() + 20; }
type Z10_S = { hash: Int; salt: Int; rounds: Int; }
fn z21() -> Int { return z20() + 21; }
enum Z10_E { O0(v: Int), O1(v: Str) }
fn z22() -> Int { return z21() + 22; }
type Z11_S = { from: Int; to: Int; weight: Float64; }
fn z23() -> Int { return z22() + 23; }
enum Z11_E { N0, N1(v: Int), N2 }
fn z24() -> Int { return z23() + 24; }
type Z12_S = { x1: Float64; y1: Float64; x2: Float64; y2: Float64; }
fn z25() -> Int { return z24() + 25; }
enum Z12_E { M0, M1, M2, M3, M4, M5 }
fn z26() -> Int { return z25() + 26; }
type Z13_S = { row: Int; col: Int; val: Float64; }
fn z27() -> Int { return z26() + 27; }
enum Z13_E { L0(v: Float64), L1(v: Int, w: Int) }
fn z28() -> Int { return z27() + 28; }
type Z14_S = { index: Int; offset: Int; len: Int; }
fn z29() -> Int { return z28() + 29; }

type Z15_S = { a: Bool; b: Char; c: Int; d: Float64; e: Str; }
type Z16_S = { alpha: Float64; beta: Float64; gamma: Float64; }
type Z17_S = { version: Int; build: Int; patch: Int; }
type Z18_S = { count: Int; sum: Int; mean: Float64; }
type Z19_S = { source: Str; dest: Str; distance: Int; }

type Mega20 = {
  f0: Bool; f1: Char; f2: Int; f3: Float64; f4: Bool;
  f5: Char; f6: Int; f7: Float64; f8: Bool; f9: Char;
  f10: Int; f11: Float64; f12: Bool; f13: Char; f14: Int;
  f15: Float64; f16: Bool; f17: Char; f18: Int; f19: Float64;
}

enum Z14_E { K0, K1(v: Str) }
enum Z15_E { J0(v: Int), J1(v: Int, w: Int), J2(v: Int, w: Int, z: Int) }

fn init_m20() -> Mega20 {
  return Mega20{
    f0: true; f1: 'Z'; f2: 1; f3: 1.0; f4: false;
    f5: 'Y'; f6: 2; f7: 2.0; f8: true; f9: 'X';
    f10: 3; f11: 3.0; f12: false; f13: 'W'; f14: 4;
    f15: 4.0; f16: true; f17: 'V'; f18: 5; f19: 5.0;
  };
}

fn sum_m20(m: Mega20) -> Int {
  var s: Int = m.f2 + m.f6 + m.f10 + m.f14 + m.f18;
  if m.f0 { s = s + 1; }
  if m.f4 { s = s + 1; }
  if m.f8 { s = s + 1; }
  if m.f12 { s = s + 1; }
  if m.f16 { s = s + 1; }
  return s;
}

fn match20(v: Int) -> Int {
  match v {
    0 => return 0; 1 => return 5; 2=>return 10; 3=>return 15; 4=>return 20;
    5=>return 25; 6=>return 30; 7=>return 35; 8=>return 40; 9=>return 45;
    10=>return 50; 11=>return 55; 12=>return 60; 13=>return 65; 14=>return 70;
    15=>return 75; 16=>return 80; 17=>return 85; 18=>return 90; 19=>return 95;
    _ => return 0;
  }
}

fn fifty() -> Int {
  var v0:Int=1;var v1:Int=1;var v2:Int=1;var v3:Int=1;var v4:Int=1;
  var v5:Int=1;var v6:Int=1;var v7:Int=1;var v8:Int=1;var v9:Int=1;
  var v10:Int=1;var v11:Int=1;var v12:Int=1;var v13:Int=1;var v14:Int=1;
  var v15:Int=1;var v16:Int=1;var v17:Int=1;var v18:Int=1;var v19:Int=1;
  var v20:Int=1;var v21:Int=1;var v22:Int=1;var v23:Int=1;var v24:Int=1;
  var v25:Int=1;var v26:Int=1;var v27:Int=1;var v28:Int=1;var v29:Int=1;
  var v30:Int=1;var v31:Int=1;var v32:Int=1;var v33:Int=1;var v34:Int=1;
  var v35:Int=1;var v36:Int=1;var v37:Int=1;var v38:Int=1;var v39:Int=1;
  var v40:Int=1;var v41:Int=1;var v42:Int=1;var v43:Int=1;var v44:Int=1;
  var v45:Int=1;var v46:Int=1;var v47:Int=1;var v48:Int=1;var v49:Int=1;
  return v0+v1+v2+v3+v4+v5+v6+v7+v8+v9+v10+v11+v12+v13+v14+v15+v16+v17+v18+v19
    +v20+v21+v22+v23+v24+v25+v26+v27+v28+v29+v30+v31+v32+v33+v34+v35+v36+v37+v38+v39
    +v40+v41+v42+v43+v44+v45+v46+v47+v48+v49;
}

fn deep10() -> Int {
  var r: Int = 0;
  if true { if true { if true { if true { if true { if true { if true { if true { if true { if true { r = 99; } } } } } } } } } }
  return r;
}

module p13_m1 {
  pub fn triple(x: Int) -> Int { return x * 3; }
}
module p13_m2 {
  use p13_m1.triple;
  pub fn triple_plus_one(x: Int) -> Int { return triple(x) + 1; }
}
module p13_m3 {
  use p13_m2.triple_plus_one;
  pub fn calc(x: Int) -> Int { return triple_plus_one(x) * 2; }
}
module p13_m4 {
  use p13_m3.calc;
  pub fn pipeline(x: Int) -> Int { return calc(x) + 5; }
}
module p13_m5 {
  use p13_m4.pipeline;
  pub fn verify(v: Int, expect: Int) -> Int {
    if pipeline(v) == expect { return 0; }
    return 1;
  }
}

fn main() -> Int {
  var chain = z29();
  if chain != 435 { return 1; }
  var mega = init_m20();
  if sum_m20(mega) != 18 { return 2; }
  if match20(7) != 35 { return 3; }
  if match20(19) != 95 { return 4; }
  if fifty() != 50 { return 5; }
  if deep10() != 99 { return 6; }
  if p13_m5.verify(1, 13) != 0 { return 7; }
  return 0;
}

