// m1827 item C (REL-20261022-1827-packages): matching a nested enum variant
// directly inside a Result pattern -- `Err(E.A(v))` -- must descend into the
// inner enum's discriminant. A single-letter enum name used to be filtered as
// a generic placeholder, so the inner check was omitted and no Err arm ran.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m1827_match_err_enum

use xiom.io;
use xiom.convert;

enum E {
  A(x: Int),
  B(y: Int),
}

fn make(tag: Int) -> Result[Int, E] {
  if tag == 1 {
    return Err(E.A(7));
  };
  return Err(E.B(9));
}

pub fn main() -> Int {
  var failures: Int = 0;
  var saw_b: Int = 0;
  match make(2) {
    Ok(_) => {
      io.println("[FAIL] unexpected Ok");
      failures = failures + 1;
    },
    Err(E.A(v)) => {
      io.println("[BUG] make(2) returned E.B(9) but matched E.A(v=" + convert.to_string(v) + ")");
      failures = failures + 1;
    },
    Err(E.B(v)) => {
      if v == 9 {
        saw_b = 1;
      } else {
        io.println("[BUG] E.B payload corrupted: " + convert.to_string(v));
        failures = failures + 1;
      };
    },
  };
  if saw_b != 1 { failures = failures + 1; }

  match make(2) {
    Ok(_) => {},
    Err(e) => {
      match e {
        E.A(v) => {
          io.println("[FAIL] control: nested match took E.A(" + convert.to_string(v) + ")");
          failures = failures + 1;
        },
        E.B(v) => {
          if v != 9 {
            io.println("[FAIL] control: E.B payload " + convert.to_string(v));
            failures = failures + 1;
          };
        },
      };
    },
  };
  if failures == 0 { io.println("m1827 direct Err(Enum.Variant) match ok"); }
  return failures;
}
