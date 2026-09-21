// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N2-01: 10-deep if-else chain -- pushes conditional nesting limit
fn main() -> Int {
  var x = 1;
  if x == 1 { if x == 1 { if x == 1 { if x == 1 { if x == 1 {
    if x == 1 { if x == 1 { if x == 1 { if x == 1 { if x == 1 {
      var r = 0;
      return r;
    } else { return 1; }
    } else { return 1; }
    } else { return 1; }
    } else { return 1; }
    } else { return 1; }
  } else { return 1; } } else { return 1; } } else { return 1; } } else { return 1; } } else { return 1; }
  return 1;
}
