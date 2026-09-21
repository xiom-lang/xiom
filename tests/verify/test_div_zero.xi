// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module test_div_zero
fn divide(a: Int, b: Int) -> Int
    requires: b != 0
    ensures: result == a / b
{
    return a / b;
}
