// m280 (REL-20261011-2230-bindings): (1) an outbound Int->ptr cast must
// survive a whole-body unsafe return -- the confined-block Copy-Out treated
// the *UInt8 tail as a Str and xiom_guard_copy_str returned a FRESH buffer
// (the caller saw a wild address; the C side rejected it). (2) a Vec[Str]
// element read off a TEMPORARY must concatenate as a string -- the concat
// verdict fell back to the i64 scalar path and printed the raw pointer as
// digits. Returns 0 only when both hold.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m280_unsafe_ptr_temp_str;

use xiom.io;
use xiom.convert;

fn ptr_ident(a: Int) -> *UInt8
  requires: true
{
  unsafe { return a as *UInt8; }
}

fn tmp_vec() -> Vec[Str] {
  var v = Vec[Str].new();
  v.push("alpha");
  v.push("beta");
  return v;
}

fn main() -> Int {
  var fails: Int = 0;
  unsafe {
    var slot = Vec[UInt8].new();
    slot.push(7 as UInt8);
    let addr = slot.as_mut_ptr() as Int;
    let p = ptr_ident(addr);
    if (p as Int) - addr != 0 { fails = fails + 1; }
  }
  let s = tmp_vec()[1] + "!";
  if s != "beta!" { fails = fails + 1; }
  // The lane's workaround shape (bind first) must stay green too.
  let v = tmp_vec();
  let s2 = v[1] + "!";
  if s2 != "beta!" { fails = fails + 1; }
  io.println("m280_unsafe_ptr_temp_str: fails=" + int_to_string(fails) + " tmp=" + s);
  return fails;
}
