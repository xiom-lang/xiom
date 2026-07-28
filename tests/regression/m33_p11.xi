// M33-P11: 50 one-line functions + 20 structs + 15 enums + deep call chain + 10 consts + 5 modules
// Pattern: fn main() -> Int { ... return 0; }

const P11_K0: Int = 0; const P11_K1: Int = 1; const P11_K2: Int = 2; const P11_K3: Int = 3; const P11_K4: Int = 4;
const P11_K5: Int = 5; const P11_K6: Int = 6; const P11_K7: Int = 7; const P11_K8: Int = 8; const P11_K9: Int = 9;

fn u00() -> Int { return 0; } fn u01() -> Int { return 1; } fn u02() -> Int { return 2; } fn u03() -> Int { return 3; } fn u04() -> Int { return 4; }
fn u05() -> Int { return 5; } fn u06() -> Int { return 6; } fn u07() -> Int { return 7; } fn u08() -> Int { return 8; } fn u09() -> Int { return 9; }
fn u10() -> Int { return 10; } fn u11() -> Int { return 11; } fn u12() -> Int { return 12; } fn u13() -> Int { return 13; } fn u14() -> Int { return 14; }
fn u15() -> Int { return 15; } fn u16() -> Int { return 16; } fn u17() -> Int { return 17; } fn u18() -> Int { return 18; } fn u19() -> Int { return 19; }
fn u20() -> Int { return 20; } fn u21() -> Int { return 21; } fn u22() -> Int { return 22; } fn u23() -> Int { return 23; } fn u24() -> Int { return 24; }
fn u25() -> Int { return 25; } fn u26() -> Int { return 26; } fn u27() -> Int { return 27; } fn u28() -> Int { return 28; } fn u29() -> Int { return 29; }
fn u30() -> Int { return 30; } fn u31() -> Int { return 31; } fn u32() -> Int { return 32; } fn u33() -> Int { return 33; } fn u34() -> Int { return 34; }
fn u35() -> Int { return 35; } fn u36() -> Int { return 36; } fn u37() -> Int { return 37; } fn u38() -> Int { return 38; } fn u39() -> Int { return 39; }
fn u40() -> Int { return 40; } fn u41() -> Int { return 41; } fn u42() -> Int { return 42; } fn u43() -> Int { return 43; } fn u44() -> Int { return 44; }
fn u45() -> Int { return 45; } fn u46() -> Int { return 46; } fn u47() -> Int { return 47; } fn u48() -> Int { return 48; } fn u49() -> Int { return 49; }

type V11_0 = { a: Int; } type V11_1 = { x: Float64; y: Float64; }
type V11_2 = { label: Str; count: Int; } type V11_3 = { flag: Bool; val: Int; extra: Float64; }
type V11_4 = { min: Int; max: Int; step: Int; } type V11_5 = { key: Str; value: Int; ok: Bool; }
type V11_6 = { r: Int; g: Int; b: Int; } type V11_7 = { x1: Float64; y1: Float64; x2: Float64; y2: Float64; }
type V11_8 = { head: Int; tail: Int; size: Int; } type V11_9 = { code: Int; msg: Str; }
type V11_10 = { version: Int; build: Int; flags: Int; } type V11_11 = { width: Int; height: Int; area: Int; }
type V11_12 = { alpha: Float64; beta: Float64; } type V11_13 = { row: Int; col: Int; data: Float64; }
type V11_14 = { hash: Int; salt: Int; } type V11_15 = { from: Int; to: Int; weight: Float64; }
type V11_16 = { total: Int; items: Int; avg: Float64; } type V11_17 = { a: Int; b: Int; c: Int; d: Int; }
type V11_18 = { index: Int; offset: Int; length: Int; } type V11_19 = { source: Str; dest: Str; port: Int; }

enum F11_0 { Alpha, Beta } enum F11_1 { One, Two, Three(v: Int) } enum F11_2 { Left, Right, Center }
enum F11_3 { Text(v: Str), Number(v: Int) } enum F11_4 { Good, Bad, Unknown } enum F11_5 { Active, Idle, Error(v: Int) }
enum F11_6 { Small, Medium, Large(v: Float64) } enum F11_7 { Open, Closed, Locked(v: Int) }
enum F11_8 { Ready, Running, Stopped } enum F11_9 { A, B, C, D, E } enum F11_10 { Success(v: Int), Failure(v: Int) }
enum F11_11 { Morning, Noon, Evening } enum F11_12 { Red, Green, Blue, Yellow }
enum F11_13 { Pass, Fail(v: Int), Skip } enum F11_14 { Hot(v: Float64), Cold(v: Float64) }
enum F11_15 { Single(v: Int), Pair(v: Int, w: Int), Triplet(v: Int, w: Int, z: Int) }

fn p11_chain0() -> Int { return P11_K0; } fn p11_chain1() -> Int { return p11_chain0() + 1; }
fn p11_chain2() -> Int { return p11_chain1() + 2; } fn p11_chain3() -> Int { return p11_chain2() + 3; }
fn p11_chain4() -> Int { return p11_chain3() + 4; } fn p11_chain5() -> Int { return p11_chain4() + 5; }
fn p11_chain6() -> Int { return p11_chain5() + 6; } fn p11_chain7() -> Int { return p11_chain6() + 7; }
fn p11_chain8() -> Int { return p11_chain7() + 8; } fn p11_chain9() -> Int { return p11_chain8() + 9; }
fn p11_chain10() -> Int { return p11_chain9() + 10; } fn p11_chain11() -> Int { return p11_chain10() + 11; }
fn p11_chain12() -> Int { return p11_chain11() + 12; } fn p11_chain13() -> Int { return p11_chain12() + 13; }
fn p11_chain14() -> Int { return p11_chain13() + 14; } fn p11_chain15() -> Int { return p11_chain14() + 15; }
fn p11_chain16() -> Int { return p11_chain15() + 16; } fn p11_chain17() -> Int { return p11_chain16() + 17; }
fn p11_chain18() -> Int { return p11_chain17() + 18; } fn p11_chain19() -> Int { return p11_chain18() + 19; }
fn p11_chain20() -> Int { return p11_chain19() + 20; }

fn sum_wide_table() -> Int {
  return u00()+u01()+u02()+u03()+u04()+u05()+u06()+u07()+u08()+u09()
    +u10()+u11()+u12()+u13()+u14()+u15()+u16()+u17()+u18()+u19()
    +u20()+u21()+u22()+u23()+u24()+u25()+u26()+u27()+u28()+u29()
    +u30()+u31()+u32()+u33()+u34()+u35()+u36()+u37()+u38()+u39()
    +u40()+u41()+u42()+u43()+u44()+u45()+u46()+u47()+u48()+u49();
}

fn match_20(v: Int) -> Int {
  match v {
    0 => return 0; 1 => return 3; 2 => return 6; 3 => return 9; 4 => return 12;
    5 => return 15; 6 => return 18; 7 => return 21; 8 => return 24; 9 => return 27;
    10 => return 30; 11 => return 33; 12 => return 36; 13 => return 39; 14 => return 42;
    15 => return 45; 16 => return 48; 17 => return 51; 18 => return 54; 19 => return 57;
    _ => return 0;
  }
}

fn deep10() -> Int {
  var x: Int = 0;
  if true { if true { if true { if true { if true { if true { if true { if true { if true { if true { x = 99; } } } } } } } } } }
  return x;
}

fn fifty_vars() -> Int {
  var a0:Int=1;var a1:Int=1;var a2:Int=1;var a3:Int=1;var a4:Int=1;var a5:Int=1;var a6:Int=1;var a7:Int=1;var a8:Int=1;var a9:Int=1;
  var a10:Int=1;var a11:Int=1;var a12:Int=1;var a13:Int=1;var a14:Int=1;var a15:Int=1;var a16:Int=1;var a17:Int=1;var a18:Int=1;var a19:Int=1;
  var a20:Int=1;var a21:Int=1;var a22:Int=1;var a23:Int=1;var a24:Int=1;var a25:Int=1;var a26:Int=1;var a27:Int=1;var a28:Int=1;var a29:Int=1;
  var a30:Int=1;var a31:Int=1;var a32:Int=1;var a33:Int=1;var a34:Int=1;var a35:Int=1;var a36:Int=1;var a37:Int=1;var a38:Int=1;var a39:Int=1;
  var a40:Int=1;var a41:Int=1;var a42:Int=1;var a43:Int=1;var a44:Int=1;var a45:Int=1;var a46:Int=1;var a47:Int=1;var a48:Int=1;var a49:Int=1;
  return a0+a1+a2+a3+a4+a5+a6+a7+a8+a9+a10+a11+a12+a13+a14+a15+a16+a17+a18+a19
    +a20+a21+a22+a23+a24+a25+a26+a27+a28+a29+a30+a31+a32+a33+a34+a35+a36+a37+a38+a39
    +a40+a41+a42+a43+a44+a45+a46+a47+a48+a49;
}

module p11_m1 { pub fn add3(a:Int,b:Int,c:Int) -> Int { return a+b+c; } }
module p11_m2 { use p11_m1.add3; pub fn mul_by_sum(a:Int,b:Int,c:Int,x:Int) -> Int { return add3(a,b,c)*x; } }
module p11_m3 { use p11_m2.mul_by_sum; pub fn process(x:Int) -> Int { return mul_by_sum(1,2,3,x); } }
module p11_m4 { use p11_m3.process; pub fn double_proc(x:Int) -> Int { return process(x)*2; } }
module p11_m5 { use p11_m4.double_proc; pub fn verify(v:Int,expect:Int) -> Int { if double_proc(v)==expect { return 0; } return 1; } }

fn main() -> Int {
  var wide = sum_wide_table();
  if wide != 1225 { return 1; }
  var chain = p11_chain20();
  if chain != 210 { return 2; }
  if match_20(7) != 21 { return 3; }
  if deep10() != 99 { return 4; }
  if fifty_vars() != 50 { return 5; }
  if p11_m5.verify(1, 12) != 0 { return 6; }
  return 0;
}
