// m342 control entry: imports the parse-clean sibling module xiom.m342vf.ok
// through the same import shape as main.xi. Must compile and run (prints 5
// then 8; exit 0) -- the escalation is inert for valid modules. Distinct
// non-zero exits mark the exact broken value.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m342_varfn_parse_surface.control;

use xiom.m342vf.ok;
use xiom.io;

fn main() -> Int {
    if ok() != 5 { return 1; }
    if calc_ok(3) != 8 { return 2; }
    println(int_to_string(ok()));
    println(int_to_string(calc_ok(3)));
    return 0;
}
