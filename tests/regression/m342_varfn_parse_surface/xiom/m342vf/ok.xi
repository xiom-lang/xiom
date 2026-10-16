// m342 control: a parse-clean sibling module with the same import shape.
// The escalation must be inert for valid modules -- control_main.xi imports
// this, calls both sibling fns, and stays green.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.m342vf.ok;

pub fn ok() -> Int {
    return 5;
}

pub fn calc_ok(n: Int) -> Int {
    var x = n + 1;
    return x * 2;
}
