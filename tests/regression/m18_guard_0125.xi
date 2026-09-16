// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0125

fn main() -> Int {
  var vi: Int = 42;
  var vf: Float64 = 3.14;
  var vb: Bool = true;
  var vc: Char = 'z';
  var vs: Str = "xiom";

  match vi {
    v if v > 0 => { }
    _ => { return 1; }
  }

  match vf {
    v if v > 1.0 => { }
    _ => { return 2; }
  }

  match vb {
    v if v == true => { }
    _ => { return 3; }
  }

  match vc {
    v if v == 'z' => { }
    _ => { return 4; }
  }

  match vs {
    v if v.len() > 0 => { return 0; }
    _ => { return 5; }
  }
}
