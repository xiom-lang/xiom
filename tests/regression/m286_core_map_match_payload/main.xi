// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m286 lock (REL-20261010-1549-stdlib-7): `o.map(to_s)` on Option[Int] with
// a Str-returning callback -- the match scrutinee payload resolved the RAW
// declared "Option[U]" -> "U", which consumers treated as a scalar (the Str
// handle byte-materialized in the arm). The substituted "Option[Str]" must
// drive the Some(s) payload binding.
module m286_core_map_match_payload.main

use xiom.string;
use xiom.convert;

fn to_s(x: Int) -> Str { return convert.int_to_string(x); }

fn main() -> Int {
  let o = Some(7);
  match o.map(to_s) {
    Some(s) => {
      if str_compare(s, "7") != 0 { return 41; }
    };
    None => { return 40; }
  }
  return 0;
}
