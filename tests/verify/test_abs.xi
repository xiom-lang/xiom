// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module test_abs

fn abs(x: Int) -> Int
    requires: x != 0
    ensures: result > 0
{
    if x < 0 {
        return -x;
    }
    return x;
}
