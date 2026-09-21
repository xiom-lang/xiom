// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module test_compose

fn square(x: Int) -> Int
    requires: x >= 0
    ensures: result >= 0
{
    return x * x;
}

fn use_square(a: Int) -> Int
    requires: a >= 0
    ensures: result >= 0
{
    return square(a);
}
