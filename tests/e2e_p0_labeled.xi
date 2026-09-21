// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// P0-3 E2E Test: Labeled break/continue
// Tests that @label break/continue targets the correct labeled loop.

use stdlib.xiom.iter;

fn main() -> Int {
    // Test 1: Labeled break from nested loop
    var outer_count = 0;
    var inner_count = 0;
    @outer: for i in range(0, 5) {
        outer_count = outer_count + 1;
        for j in range(0, 10) {
            inner_count = inner_count + 1;
            if j >= 2 { break @outer; }
        }
        // This should NOT be reached after break @outer
        outer_count = 999;  
    }
    // outer_count should be 1 (only i=0), inner_count should be 3 (j=0,1,2)
    if outer_count != 1 { return 1; }
    if inner_count != 3 { return 2; }

    // Test 2: Labeled continue to outer loop  
    var skipped = 0;
    @outer2: for i in range(0, 3) {
        for j in range(0, 5) {
            if j == 1 { continue @outer2; }
            skipped = skipped + 1;
        }
    }
    if skipped != 3 { return 3; }

    // Test 3: Unlabeled break still targets innermost loop
    var found = 0;
    for i in range(0, 5) {
        for j in range(0, 10) {
            found = found + 1;
            break;  // should break inner loop only
        }
    }
    // found = 5 (one per outer iteration)
    if found != 5 { return 4; }

    return 0;
}
