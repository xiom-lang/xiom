// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// P0-1 E2E Test: for..in iteration
// Tests that for..in actually iterates, not just runs body once.

use stdlib.xiom.iter;

fn main() -> Int {
    // Test 1: Range iteration (0..5)
    var sum = 0;
    for i in range(0, 5) {
        sum = sum + i;
    }
    // sum should be 0+1+2+3+4 = 10
    if sum != 10 { return 1; }

    // Test 2: Empty range
    var count = 0;
    for i in range(0, 0) {
        count = count + 1;
    }
    if count != 0 { return 2; }

    // Test 3: Single element range
    var single = 0;
    for i in range(5, 6) {
        single = single + i;
    }
    if single != 5 { return 3; }

    // Test 4: Using break inside for..in
    var broke = 0;
    for i in range(0, 100) {
        broke = broke + 1;
        if i >= 2 { break; }
    }
    // Should break after i=2 (3 iterations: i=0,1,2)
    if broke != 3 { return 4; }

    // Test 5: Using continue inside for..in
    var continued = 0;
    for i in range(0, 5) {
        if i == 2 { continue; }
        continued = continued + 1;
    }
    // Should skip i=2, so 4 iterations
    if continued != 4 { return 5; }

    return 0;
}
