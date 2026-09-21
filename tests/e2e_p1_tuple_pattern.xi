// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// P1-2 E2E Test: Tuple patterns in match
// Tests match with tuple destructuring patterns.

fn main() -> Int {
    // Test 1: Basic tuple pattern match
    var pair = (10, 20);
    var sum = 0;
    match pair {
        (a, b) => { sum = a + b; }
    }
    if sum != 30 { return 1; }

    // Test 2: Tuple pattern with wildcard
    var triple = (1, 2, 3);
    var first = 0;
    match triple {
        (a, _, _) => { first = a; }
    }
    if first != 1 { return 2; }

    // Test 3: Tuple pattern with literal value
    var pair2 = (42, 99);
    var found = false;
    match pair2 {
        (42, b) => { found = true; }
        _ => {}
    }
    if !found { return 3; }

    // Test 4: Match with OR tuple pattern
    var val = (7, 0);
    var hit = 0;
    match val {
        (0, _) | (7, _) => { hit = 7; }
        _ => {}
    }
    if hit != 7 { return 4; }

    return 0;
}
