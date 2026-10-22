// m1827 item B consumer (see m1827_vec_in_tuple_lib.xi).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m1827_vec_in_tuple_main

use m1827_vec_in_tuple_lib;
use xiom.io;
use xiom.convert;

pub fn main() -> Int {
  var failures: Int = 0;
  match pair() {
    Ok((_h, v)) => {
      if v.len() != 2 {
        io.println("cross-module nominal len=" + convert.int_to_string(v.len()));
        failures = failures + 1;
      } else if v[0] != 11 || v[1] != 22 {
        io.println("cross-module nominal contents corrupted");
        failures = failures + 1;
      };
    }
    Err(_) => {
      io.println("unexpected Err");
      failures = failures + 1;
    }
  }
  if failures == 0 { io.println("m1827 cross-module Vec-in-tuple ok"); }
  return failures;
}
