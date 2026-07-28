// M33-P12: Interleaved fn/type/enum/const + 20-struct + 15-enum + deep chain + big match
// Pattern: fn main() -> Int { ... return 0; }

fn p12_a00() -> Int { return 0; }
type P12_T00 = { a: Int; }
fn p12_a01() -> Int { return p12_a00() + 1; }
enum P12_E00 { A0, A1 }
fn p12_a02() -> Int { return p12_a01() + 2; }
type P12_T01 = { x: Float64; y: Float64; }
fn p12_a03() -> Int { return p12_a02() + 3; }
enum P12_E01 { B0(v: Int), B1 }
fn p12_a04() -> Int { return p12_a03() + 4; }
type P12_T02 = { label: Str; count: Int; }
fn p12_a05() -> Int { return p12_a04() + 5; }
const P12_C0: Int = 10;
enum P12_E02 { C0, C1, C2 }
fn p12_a06() -> Int { return p12_a05() + 6; }
type P12_T03 = { flag: Bool; val: Int; ok: Bool; }
fn p12_a07() -> Int { return p12_a06() + 7; }
enum P12_E03 { D0(v: Str), D1 }
fn p12_a08() -> Int { return p12_a07() + 8; }
type P12_T04 = { min: Int; max: Int; step: Int; }
fn p12_a09() -> Int { return p12_a08() + 9; }
const P12_C1: Int = 20;
enum P12_E04 { E0, E1, E2, E3 }
fn p12_a10() -> Int { return p12_a09() + 10; }
type P12_T05 = { key: Str; val: Int; ttl: Int; }
fn p12_a11() -> Int { return p12_a10() + 11; }
enum P12_E05 { F0(v: Float64), F1, F2 }
fn p12_a12() -> Int { return p12_a11() + 12; }
type P12_T06 = { r: Int; g: Int; b: Int; a: Int; }
fn p12_a13() -> Int { return p12_a12() + 13; }
enum P12_E06 { G0, G1(v: Int, w: Int) }
fn p12_a14() -> Int { return p12_a13() + 14; }
type P12_T07 = { width: Int; height: Int; depth: Int; }
fn p12_a15() -> Int { return p12_a14() + 15; }
const P12_C2: Int = 30;
enum P12_E07 { Open, Closed, Locked(v: Int) }
fn p12_a16() -> Int { return p12_a15() + 16; }
type P12_T08 = { row: Int; col: Int; data: Float64; }
fn p12_a17() -> Int { return p12_a16() + 17; }
enum P12_E08 { Ready, Running, Stopped, Error(v: Str) }
fn p12_a18() -> Int { return p12_a17() + 18; }
type P12_T09 = { code: Int; msg: Str; }
fn p12_a19() -> Int { return p12_a18() + 19; }
const P12_C3: Int = 40;
enum P12_E09 { A, B, C, D, E }
fn p12_a20() -> Int { return p12_a19() + 20; }
type P12_T10 = { alpha: Float64; beta: Float64; gamma: Float64; }
fn p12_a21() -> Int { return p12_a20() + 21; }
enum P12_E10 { Single(v: Int), Pair(v: Int, w: Int) }
fn p12_a22() -> Int { return p12_a21() + 22; }
type P12_T11 = { hash: Int; salt: Int; rounds: Int; }
fn p12_a23() -> Int { return p12_a22() + 23; }
const P12_C4: Int = 50;
enum P12_E11 { Success(v: Int), Failure(v: Int, s: Str) }
fn p12_a24() -> Int { return p12_a23() + 24; }
type P12_T12 = { total: Int; items: Int; rate: Float64; }
fn p12_a25() -> Int { return p12_a24() + 25; }
enum P12_E12 { Pass, Fail(v: Int), Skip }
fn p12_a26() -> Int { return p12_a25() + 26; }
type P12_T13 = { version: Int; build: Int; flags: Int; }
fn p12_a27() -> Int { return p12_a26() + 27; }
const P12_C5: Int = 60;
enum P12_E13 { Morning, Noon, Evening, Night }
fn p12_a28() -> Int { return p12_a27() + 28; }
type P12_T14 = { from: Int; to: Int; weight: Float64; }
fn p12_a29() -> Int { return p12_a28() + 29; }

type P12_T15 = { x1: Float64; y1: Float64; x2: Float64; y2: Float64; }
type P12_T16 = { count: Int; sum: Int; mean: Float64; variance: Float64; }
type P12_T17 = { index: Int; offset: Int; length: Int; }
type P12_T18 = { a: Bool; b: Char; c: Int; d: Float64; e: Str; }
type P12_T19 = { head: Int; next: Int; prev: Int; data: Float64; }
type P12_Mega = {
  m0: Bool; m1: Char; m2: Int; m3: Float64; m4: Bool;
  m5: Char; m6: Int; m7: Float64; m8: Bool; m9: Char;
  m10: Int; m11: Float64; m12: Bool; m13: Char; m14: Int;
  m15: Float64; m16: Bool; m17: Char; m18: Int; m19: Float64;
}

enum P12_E14 { Animal(v: Str, legs: Int), Plant, Fungus }
enum P12_E15 { High(v: Float64), Medium(v: Float64), Low(v: Float64) }

fn init_mega() -> P12_Mega {
  return P12_Mega{
    m0: true; m1: 'A'; m2: 1; m3: 1.0; m4: false;
    m5: 'B'; m6: 2; m7: 2.0; m8: true; m9: 'C';
    m10: 3; m11: 3.0; m12: false; m13: 'D'; m14: 4;
    m15: 4.0; m16: true; m17: 'E'; m18: 5; m19: 5.0;
  };
}

fn sum_mega(m: P12_Mega) -> Int {
  var s: Int = m.m2 + m.m6 + m.m10 + m.m14 + m.m18;
  if m.m0 { s = s + 1; }
  if m.m4 { s = s + 1; }
  if m.m8 { s = s + 1; }
  if m.m12 { s = s + 1; }
  if m.m16 { s = s + 1; }
  return s;
}

fn big_match(val: Int) -> Int {
  match val {
    0 => return 0; 1 => return 1; 2 => return 4; 3 => return 9; 4 => return 16;
    5 => return 25; 6 => return 36; 7 => return 49; 8 => return 64; 9 => return 81;
    10 => return 100; 11 => return 121; 12 => return 144; 13 => return 169; 14 => return 196;
    15 => return 225; 16 => return 256; 17 => return 289; 18 => return 324; 19 => return 361;
    _ => return 0;
  }
}

fn fifty_locals() -> Int {
  var x00:Int=1;var x01:Int=1;var x02:Int=1;var x03:Int=1;var x04:Int=1;
  var x05:Int=1;var x06:Int=1;var x07:Int=1;var x08:Int=1;var x09:Int=1;
  var x10:Int=1;var x11:Int=1;var x12:Int=1;var x13:Int=1;var x14:Int=1;
  var x15:Int=1;var x16:Int=1;var x17:Int=1;var x18:Int=1;var x19:Int=1;
  var x20:Int=1;var x21:Int=1;var x22:Int=1;var x23:Int=1;var x24:Int=1;
  var x25:Int=1;var x26:Int=1;var x27:Int=1;var x28:Int=1;var x29:Int=1;
  var x30:Int=1;var x31:Int=1;var x32:Int=1;var x33:Int=1;var x34:Int=1;
  var x35:Int=1;var x36:Int=1;var x37:Int=1;var x38:Int=1;var x39:Int=1;
  var x40:Int=1;var x41:Int=1;var x42:Int=1;var x43:Int=1;var x44:Int=1;
  var x45:Int=1;var x46:Int=1;var x47:Int=1;var x48:Int=1;var x49:Int=1;
  return x00+x01+x02+x03+x04+x05+x06+x07+x08+x09+x10+x11+x12+x13+x14+x15+x16+x17+x18+x19
    +x20+x21+x22+x23+x24+x25+x26+x27+x28+x29+x30+x31+x32+x33+x34+x35+x36+x37+x38+x39
    +x40+x41+x42+x43+x44+x45+x46+x47+x48+x49;
}

fn deep10() -> Int { var r:Int=0; if true { if true { if true { if true { if true { if true { if true { if true { if true { if true { r=42; } } } } } } } } } } return r; }

module p12_m_a {
  pub type MA_T = { v: Int; }
  pub fn add(x: Int, y: Int) -> Int { return x + y; }
}
module p12_m_b {
  use p12_m_a.add;
  pub fn sum3(a: Int, b: Int, c: Int) -> Int { return add(add(a, b), c); }
}
module p12_m_c {
  use p12_m_b.sum3;
  pub fn times2(a: Int, b: Int, c: Int) -> Int { return sum3(a, b, c) * 2; }
}
module p12_m_d {
  use p12_m_c.times2;
  pub fn pipeline(x: Int) -> Int { return times2(x, x, x); }
}
module p12_m_e {
  use p12_m_d.pipeline;
  pub fn validate(x: Int, expect: Int) -> Int {
    if pipeline(x) == expect { return 0; }
    return 1;
  }
}

fn main() -> Int {
  var chain_val = p12_a29();
  if chain_val != 435 { return 1; }
  var mega = init_mega();
  if sum_mega(mega) != 18 { return 2; }
  if big_match(8) != 64 { return 3; }
  if fifty_locals() != 50 { return 4; }
  if deep10() != 42 { return 5; }
  if p12_m_e.validate(2, 12) != 0 { return 6; }
  return 0;
}
