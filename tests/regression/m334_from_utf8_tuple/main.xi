// m334 (REL-20261015-1941-packages): an INLINE `Str::from_utf8(...)` call as
// the first component of a tuple RESULT mis-named the tuple element -- both
// tuple-element namers derive the element name through infer_llvm_type,
// which reported i64 for the builtin Str intercepts (they compile to i8* but
// have no fn-registry entry), so the literal built a 16-byte
// Tuple__Int__Int while the fn signature used Tuple__Str__Int -> clang
// "'%tmp' defined with type '%struct.Tuple__Int__Int' but expected
// '%struct.Tuple__Str__Int'." Binding the call to a local first hid the bug
// via local_xiom_types. Red arms: A (the reported Result Ok payload shape)
// and D (a direct tuple return, same class via `ret`). Controls: the
// let-bound form (B) and the Err arm (C).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m334_from_utf8_tuple;

use xiom.io;
use xiom.convert;

pub type PgError = {
  kind: Int;
  message: Str;
}

fn err_protocol(message: Str) -> PgError {
  return PgError{ kind: 2; message: message };
}

fn make_vec() -> Vec[UInt8] {
  var out = Vec[UInt8].new();
  out.push(104);
  out.push(105);
  return out;
}

// A: the reported red shape -- inline Str::from_utf8 as the FIRST component
// of the Ok payload tuple.
fn pair_inline() -> Result[(Str, Int), PgError] {
  var out = make_vec();
  return Ok((Str::from_utf8(out), 1));
}

// B: control -- let-bound form (green pre-fix, must stay green).
fn pair_bound() -> Result[(Str, Int), PgError] {
  var out = make_vec();
  let s = Str::from_utf8(out);
  return Ok((s, 2));
}

// C: control -- Err arm.
fn pair_err() -> Result[(Str, Int), PgError] {
  return Err(err_protocol("nope"));
}

// D: second red arm -- direct (non-Result) tuple return with an inline
// producer (pre-fix, clang rejected the `ret` with the same mismatch).
fn pair_direct() -> (Str, Int) {
  var out = make_vec();
  return (Str::from_utf8(out), 3);
}

pub fn main() -> Int {
  var fails: Int = 0;

  let ra = pair_inline();
  match ra {
    Err(_) => {
      fails = fails + 1;
      io.println("  [A] inline pair returned Err");
    }
    Ok(pair) => {
      if pair.0 != "hi" {
        fails = fails + 1;
        io.println("  [A] pair.0 wrong: " + pair.0);
      }
      if pair.1 != 1 {
        fails = fails + 1;
        io.println("  [A] pair.1 wrong: " + convert.int_to_string(pair.1));
      }
    }
  }

  let rb = pair_bound();
  match rb {
    Err(_) => {
      fails = fails + 1;
      io.println("  [B] bound pair returned Err");
    }
    Ok(pair) => {
      if pair.0 != "hi" {
        fails = fails + 1;
        io.println("  [B] pair.0 wrong: " + pair.0);
      }
      if pair.1 != 2 {
        fails = fails + 1;
        io.println("  [B] pair.1 wrong: " + convert.int_to_string(pair.1));
      }
    }
  }

  let rc2 = pair_err();
  match rc2 {
    Ok(_) => {
      fails = fails + 1;
      io.println("  [C] pair_err returned Ok");
    }
    Err(e) => {
      if e.kind != 2 {
        fails = fails + 1;
        io.println("  [C] Err kind wrong: " + convert.int_to_string(e.kind));
      }
    }
  }

  let td = pair_direct();
  if td.0 != "hi" {
    fails = fails + 1;
    io.println("  [D] td.0 wrong: " + td.0);
  }
  if td.1 != 3 {
    fails = fails + 1;
    io.println("  [D] td.1 wrong: " + convert.int_to_string(td.1));
  }

  io.println("m334_from_utf8_tuple: " + convert.int_to_string(fails) + " failing checks");
  return fails;
}
