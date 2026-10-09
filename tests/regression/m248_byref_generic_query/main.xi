// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m248 (stdlib byref-generic finding): `.is_some` / `.is_ok` / `.is_none`
// / `.is_err` on a POINTER-to-Option/Result local (`o: &Option[T]` in the
// generic query bodies core.option_is_some et al.) must read the tag
// THROUGH the pointer. Pre-fix the pseudo-field handler was value-gated,
// so the generic bodies emitted a bare constant 0 and
// core.option_is_some(&Some(4)) returned false.
module m248_byref_generic_query

use xiom.core;

fn main() -> Int {
  var o: Option[Int] = Some(4);
  if o.is_some == false { return 1; }
  if core.option_is_some(&o) == false { return 2; }
  if core.option_is_none(&o) == true { return 3; }

  var n: Option[Int] = None;
  if core.option_is_some(&n) == true { return 4; }
  if core.option_is_none(&n) == false { return 5; }

  var r: Result[Int, Str] = Ok(4);
  if core.result_is_ok(&r) == false { return 6; }
  if core.result_is_err(&r) == true { return 7; }

  var e: Result[Int, Str] = Err("no");
  if core.result_is_ok(&e) == true { return 8; }
  if core.result_is_err(&e) == false { return 9; }

  return 0;
}
