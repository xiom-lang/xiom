// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// R64 regression fixture: the verifier must register the bundled stdlib so
// this file type-checks. Proof strength is not the point here (math.shl is
// uninterpreted in the SMT encoding) -- resolving `math` is.
module test_stdlib_import

use xiom.math;

fn dbl(x: Int) -> Int
    requires: x >= 0
{
    return math.shl(x, 1);
}
