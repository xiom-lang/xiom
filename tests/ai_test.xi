// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn buggy_divide(a: Int, b: Int) -> Int
    requires: b != 0
    ensures: result > 0
{
    return a / b;
}
