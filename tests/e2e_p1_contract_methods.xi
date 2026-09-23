// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// P1-4 E2E Test: Contract collection methods
// Tests that is_sorted(), contains() are recognized by the checker
// and compile correctly. m119 split the runtime-intrinsic lowering into an
// inline scan over the Vec header; the values are asserted here (the old
// intrinsic answered garbage and could walk arbitrary memory -- the Windows CI
// runner took an access violation on this fixture).

fn main() -> Int {
    var arr = [1, 2, 3, 4, 5];

    // Test 1: is_sorted() -- a sorted Vec must report true.
    if !arr.is_sorted() { return 1; }

    // Test 2: contains() with a present element.
    if !arr.contains(3) { return 2; }

    // Test 3: contains() with a missing element.
    if arr.contains(9) { return 3; }

    return 0;
}
