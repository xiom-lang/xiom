// m336 (REL-20261015-2022-orbitdb; C-ORBIT-12): a tuple with an ENUM element
// inside a Result whose ERROR type is ALSO an enum (`Result[(T8Val, Int),
// T8Err]`) miscompiled. The signature namer (concrete_type_for's Result arm)
// lowered the tuple payload via type_from_ast -- a BARE Tuple__T8Val__Int --
// while the construction namers (expr.rs tuple-literal path + m305) built the
// module-qualified Tuple__m336_enum_tuple_enum_error.T8Val__Int. Two names for
// one tuple struct: the Ok lowering stored the qualified value into the
// bare-named Result field and clang hard-failed
// (`%tmpN defined with type ... but expected ...`).
//
// Red on v0.64.5, local main and the m334 branch; the enum ERROR type is the
// differentiator (with an Int error neither payload concretizes and the
// generic %struct.Result boxes the tuple -- the split stays invisible).
// This fixture ports the ORBITDB repro and adds the Int-error control inline:
// the enum-error path must decode sum=110 and the Int-error control must
// decode the same sum=110. Also gates the Err arm.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m336_enum_tuple_enum_error;

use xiom.io;
use xiom.convert;

pub type T8Val = enum { Vn, Vi(x: Int) }
pub type T8Err = enum { Bad, Worse }

// Red arm: enum error type -> concrete Result__Tuple__<mod>.T8Val__Int__T8Err.
fn read_ee(k: Int) -> Result[(T8Val, Int), T8Err] {
  if k == 0 {
    let x = 7;
    return Ok((T8Val.Vi(x), 9));
  }
  if k == 3 {
    return Err(T8Err.Worse);
  }
  return Ok((T8Val.Vn, 1));
}

// Int-error control: stays on the generic %struct.Result (boxed payload), the
// same shape that was green pre-fix -- must keep decoding 110.
fn read_int(k: Int) -> Result[(T8Val, Int), Int] {
  if k == 0 {
    let x = 7;
    return Ok((T8Val.Vi(x), 9));
  }
  return Ok((T8Val.Vn, 1));
}

fn kindof(v: T8Val) -> Int {
  match v {
    Vn => { return 0; }
    Vi(_) => { return 1; }
  }
}

fn decode_ee(k: Int) -> Int {
  match read_ee(k) {
    Err(_) => { return -1; }
    Ok(p) => {
      let v = p.0;
      let len = p.1;
      return kindof(v) * 100 + len;
    }
  }
}

fn decode_int(k: Int) -> Int {
  match read_int(k) {
    Err(_) => { return -1; }
    Ok(p) => {
      let v = p.0;
      let len = p.1;
      return kindof(v) * 100 + len;
    }
  }
}

fn main() -> Int {
  var fails = 0;
  var sum = 0;
  sum = sum + decode_ee(0);
  sum = sum + decode_ee(1);
  if sum != 110 {
    fails = fails + 1;
    io.println("  [ee] sum wrong: " + convert.int_to_string(sum) + " (expected 110)");
  }
  if decode_ee(3) != -1 {
    fails = fails + 1;
    io.println("  [ee] Err arm wrong");
  }
  var isum = 0;
  isum = isum + decode_int(0);
  isum = isum + decode_int(1);
  if isum != 110 {
    fails = fails + 1;
    io.println("  [int] sum wrong: " + convert.int_to_string(isum) + " (expected 110)");
  }
  if fails == 0 {
    io.println("m336: sum=110 control=110 done");
  } else {
    io.println("m336: " + convert.int_to_string(fails) + " failing checks");
  }
  return fails;
}
