// M33-P01: Large program stress -- deep call chain, 20 structs, 15 enums, 10 consts, 5 modules
// Pattern: fn main() -> Int { ... return 0; }
// Stress: 30-fn call chain, deep call chain fn0->fn1->...->fn20, all primitives struct, interleaved decls

const C_START: Int = 0;
const D0: Int = 0;

fn deep00() -> Int { return D0; }
fn deep01() -> Int { return 1 + deep00(); }
fn deep02() -> Int { return 2 + deep01(); }
fn deep03() -> Int { return 3 + deep02(); }
fn deep04() -> Int { return 4 + deep03(); }
fn deep05() -> Int { return 5 + deep04(); }
fn deep06() -> Int { return 6 + deep05(); }
fn deep07() -> Int { return 7 + deep06(); }
fn deep08() -> Int { return 8 + deep07(); }
fn deep09() -> Int { return 9 + deep08(); }
fn deep10() -> Int { return 10 + deep09(); }
fn deep11() -> Int { return 11 + deep10(); }
fn deep12() -> Int { return 12 + deep11(); }
fn deep13() -> Int { return 13 + deep12(); }
fn deep14() -> Int { return 14 + deep13(); }
fn deep15() -> Int { return 15 + deep14(); }
fn deep16() -> Int { return 16 + deep15(); }
fn deep17() -> Int { return 17 + deep16(); }
fn deep18() -> Int { return 18 + deep17(); }
fn deep19() -> Int { return 19 + deep18(); }
fn deep20() -> Int { return 20 + deep19(); }

const D1: Int = 1;

type T00 = { id: Int; }
fn chain00() -> Int { return C_START; }

enum E00 { Void }
fn chain01() -> Int { return chain00() + 1; }

type T01 = { x: Float64; y: Float64; }
enum E01 { A, B, C }
fn chain02() -> Int { return chain01() + 2; }

type T02 = { name: Str; count: Int; }
enum E02 { Low(v: Int), High(v: Int) }
fn chain03() -> Int { return chain02() + 3; }

type T03 = { flag: Bool; score: Float64; }
enum E03 { One, Two, Three, Four }
fn chain04() -> Int { return chain03() + 4; }

type T04 = { ch: Char; val: Int; extra: Float64; }
enum E04 { Start, Middle(v: Float64), End }
fn chain05() -> Int { return chain04() + 5; }

type T05 = { a: Int; b: Int; c: Int; d: Float64; }
enum E05 { Min, Max(v: Int, w: Int) }
fn chain06() -> Int { return chain05() + 6; }

type T06 = { inner: T00; label: Str; }
enum E06 { North, South, East, West, Center(v: Int) }
fn chain07() -> Int { return chain06() + 7; }

type T07 = { pair: T01; active: Bool; }
enum E07 { Alpha, Beta(v: Str), Gamma }
fn chain08() -> Int { return chain07() + 8; }

type T08 = { items: Int; cost: Float64; ok: Bool; }
enum E08 { Found(v: Int, w: Float64), NotFound }
fn chain09() -> Int { return chain08() + 9; }

type T09 = { key: Char; value: Int; next: Int; }
enum E09 { Yes(v: Bool), No, Maybe(v: Int) }
fn chain10() -> Int { return chain09() + 10; }

type T10 = { head: Int; tail: Int; size: Int; }
enum E10 { Red, Green, Blue, Alpha, BetaGamma }
fn chain11() -> Int { return chain10() + 11; }

type T11 = { x1: Float64; y1: Float64; x2: Float64; y2: Float64; }
enum E11 { Sun, Moon(v: Float64), Star(v: Int) }
fn chain12() -> Int { return chain11() + 12; }

type T12 = { base: T03; offset: Int; }
enum E12 { Cat(v: Str), Dog, Bird(v: Int) }
fn chain13() -> Int { return chain12() + 13; }

type T13 = { row: Int; col: Int; data: Float64; }
enum E13 { Input(v: Char), Output, Error(v: Int, w: Str) }
fn chain14() -> Int { return chain13() + 14; }

const D2: Int = 2;
const D3: Int = 3;

type T14 = { begin: Int; end: Int; step: Int; }
enum E14 { Open(v: Str), Closed, Locked(v: Int, s: Str) }
fn chain15() -> Int { return chain14() + 15; }

type T15 = { ts: Int; level: Int; message: Str; }
fn chain16() -> Int { return chain15() + 16; }

const D4: Int = 4;

type T16 = { id: Int; owner: Str; created: Int; }
fn chain17() -> Int { return chain16() + 17; }

type T17 = { min: Float64; max: Float64; avg: Float64; }
fn chain18() -> Int { return chain17() + 18; }

type T18 = { code: Int; text: Str; severity: Int; }
fn chain19() -> Int { return chain18() + 19; }

type T19 = { sum: Int; count: Int; product: Float64; valid: Bool; }

const D5: Int = 5;
const D6: Int = 6;

fn chain20() -> Int { return chain19() + 20; }
fn chain21() -> Int { return chain20() + 21; }
fn chain22() -> Int { return chain21() + 22; }
fn chain23() -> Int { return chain22() + 23; }
fn chain24() -> Int { return chain23() + 24; }
fn chain25() -> Int { return chain24() + 25; }
fn chain26() -> Int { return chain25() + 26; }
fn chain27() -> Int { return chain26() + 27; }
fn chain28() -> Int { return chain27() + 28; }
fn chain29() -> Int { return chain28() + 29; }

const D7: Int = 7;

module types_a {
  pub type PA = { id: Int; label: Str; }
  pub fn make_pa() -> PA { return PA{ id: 7; label: "ok"; }; }
}

module types_b {
  pub type PB = { x: Float64; y: Float64; z: Float64; }
  pub fn zero_pb() -> PB { return PB{ x: 0.0; y: 0.0; z: 0.0; }; }
}

module ops_c {
  pub fn add3(a: Int, b: Int, c: Int) -> Int { return a + b + c; }
  pub fn mul2(a: Int, b: Int) -> Int { return a * b; }
}

module ops_d {
  pub type PD = { total: Int; rate: Float64; }
  pub fn calc_pd() -> PD {
    var t: Int = ops_c.add3(1, 2, 3);
    var r: Float64 = 2.5;
    return PD{ total: t; rate: r; };
  }
}

module check_e {
  pub fn verify(v: Int, expected: Int) -> Int {
    if v == expected { return 0; }
    return 1;
  }
}

const D8: Int = 8;
const D9: Int = 9;
const D10: Int = 10;

type MegaStruct = {
  f_int: Int;
  f_float: Float64;
  f_bool: Bool;
  f_char: Char;
  f_int8: Int;
  f_int16: Int;
  f_int32: Int;
  f_int64: Int;
  f_float32: Float64;
  f_float64: Float64;
  f_uint8: Int;
  f_uint16: Int;
  f_uint32: Int;
  f_uint64: Int;
  f_str: Str;
  f_val_a: Int;
  f_val_b: Float64;
  f_flag_x: Bool;
  f_ch_y: Char;
  f_extra: Int;
}

fn init_mega() -> MegaStruct {
  return MegaStruct{
    f_int: 10;
    f_float: 3.14;
    f_bool: true;
    f_char: 'X';
    f_int8: 1;
    f_int16: 2;
    f_int32: 3;
    f_int64: 4;
    f_float32: 1.5;
    f_float64: 2.5;
    f_uint8: 5;
    f_uint16: 6;
    f_uint32: 7;
    f_uint64: 8;
    f_str: "mega";
    f_val_a: 9;
    f_val_b: 10.0;
    f_flag_x: false;
    f_ch_y: 'Y';
    f_extra: 11;
  };
}

fn mega_sum(m: MegaStruct) -> Int {
  var s: Int = m.f_int + m.f_int8 + m.f_int16 + m.f_int32 + m.f_int64;
  s = s + m.f_uint8 + m.f_uint16 + m.f_uint32 + m.f_uint64;
  s = s + m.f_val_a + m.f_extra;
  return s;
}

fn deep_long_fn() -> Int {
  var a0: Int = 1;
  var a1: Int = 2;
  var a2: Int = 3;
  var a3: Int = 4;
  var a4: Int = 5;
  var a5: Int = 6;
  var a6: Int = 7;
  var a7: Int = 8;
  var a8: Int = 9;
  var a9: Int = 10;
  var a10: Int = 11;
  var a11: Int = 12;
  var a12: Int = 13;
  var a13: Int = 14;
  var a14: Int = 15;
  var a15: Int = 16;
  var a16: Int = 17;
  var a17: Int = 18;
  var a18: Int = 19;
  var a19: Int = 20;
  var a20: Int = 21;
  var a21: Int = 22;
  var a22: Int = 23;
  var a23: Int = 24;
  var a24: Int = 25;
  var a25: Int = 26;
  var a26: Int = 27;
  var a27: Int = 28;
  var a28: Int = 29;
  var a29: Int = 30;
  var a30: Int = 31;
  var a31: Int = 32;
  var a32: Int = 33;
  var a33: Int = 34;
  var a34: Int = 35;
  var a35: Int = 36;
  var a36: Int = 37;
  var a37: Int = 38;
  var a38: Int = 39;
  var a39: Int = 40;
  var a40: Int = 41;
  var a41: Int = 42;
  var a42: Int = 43;
  var a43: Int = 44;
  var a44: Int = 45;
  var a45: Int = 46;
  var a46: Int = 47;
  var a47: Int = 48;
  var a48: Int = 49;
  var a49: Int = 50;
  return a0 + a1 + a2 + a3 + a4 + a5 + a6 + a7 + a8 + a9
    + a10 + a11 + a12 + a13 + a14 + a15 + a16 + a17 + a18 + a19
    + a20 + a21 + a22 + a23 + a24 + a25 + a26 + a27 + a28 + a29
    + a30 + a31 + a32 + a33 + a34 + a35 + a36 + a37 + a38 + a39
    + a40 + a41 + a42 + a43 + a44 + a45 + a46 + a47 + a48 + a49;
}

fn deep_nested_blocks() -> Int {
  var level: Int = 0;
  if true {
    level = level + 1;
    if true {
      level = level + 1;
      if true {
        level = level + 1;
        if true {
          level = level + 1;
          if true {
            level = level + 1;
            if true {
              level = level + 1;
              if true {
                level = level + 1;
                if true {
                  level = level + 1;
                  if true {
                    level = level + 1;
                    if true {
                      level = level + 1;
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
  return level;
}

fn large_match(val: Int) -> Int {
  match val {
    0 => return 0,
    1 => return 10,
    2 => return 20,
    3 => return 30,
    4 => return 40,
    5 => return 50,
    6 => return 60,
    7 => return 70,
    8 => return 80,
    9 => return 90,
    10 => return 100,
    11 => return 110,
    12 => return 120,
    13 => return 130,
    14 => return 140,
    15 => return 150,
    16 => return 160,
    17 => return 170,
    18 => return 180,
    19 => return 190,
    _ => return -1,
  }
}

fn use_module_types() -> Int {
  var pa = types_a.make_pa();
  var pb = types_b.zero_pb();
  if pa.id != 7 { return 1; }
  if pb.x != 0.0 { return 2; }
  return 0;
}

fn mega_sum_test() -> Int {
  var m_size: Int = mega_sum(init_mega());
  if m_size != 66 { return 1; }
  return 0;
}

fn use_enum_extract(e: E01) -> Int {
  match e {
    E01.A => return 1,
    E01.B => return 2,
    E01.C => return 3,
  }
}

fn use_enum_payload(e: E02) -> Int {
  match e {
    E02.Low(v) => return v,
    E02.High(v) => return v * 10,
  }
}

fn interleaved_verify() -> Int {
  var c26: Int = chain26();
  if c26 != 351 { return 1; }
  var c29: Int = chain29();
  if c29 != 435 { return 2; }
  var m_size: Int = mega_sum(init_mega());
  if m_size != 66 { return 3; }
  var locals_sum: Int = deep_long_fn();
  if locals_sum != 1275 { return 4; }
  var nest: Int = deep_nested_blocks();
  if nest != 10 { return 5; }
  var lm: Int = large_match(7);
  if lm != 70 { return 6; }
  return 0;
}

fn main() -> Int {
  var deep = deep20();
  if deep != 210 { return 1; }
  if use_enum_extract(E01.B) != 2 { return 2; }
  if use_enum_payload(E02.High(5)) != 50 { return 3; }
  if interleaved_verify() != 0 { return 4; }
  return 0;
}
