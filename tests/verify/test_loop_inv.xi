// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module test_loop_inv

fn sum_to(n: Int) -> Int
    requires: n >= 0
    ensures: result >= 0
{
    var i = 0;
    var total = 0;
    while i < n invariant: total >= 0
    {
        total = total + i;
        i = i + 1;
    }
    return total;
}
