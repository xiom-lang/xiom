// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// P1-1 E2E Test: Struct patterns in match
// Tests match with struct destructuring patterns.

type Point = { x: Int; y: Int; }

fn main() -> Int {
    // Test 1: Basic struct pattern match
    var p = Point { x: 3; y: 4 };
    var sum = 0;
    match p {
        Point { x, y } => { sum = x + y; }
    }
    if sum != 7 { return 1; }

    // Test 2: Struct pattern with bound field
    var p2 = Point { x: 10; y: 20 };
    var px = 0;
    match p2 {
        Point { x, y: _ } => { px = x; }
    }
    if px != 10 { return 2; }

    // Test 3: Struct pattern matching a literal value field
    var p3 = Point { x: 5; y: 6 };
    var matched = false;
    match p3 {
        Point { x: 5, y } => { matched = true; }
        _ => {}
    }
    if !matched { return 3; }

    // Test 4: Struct pattern bind all fields
    var p4 = Point { x: 7; y: 8 };
    var dx = 0;
    var dy = 0;
    match p4 {
        Point { x, y } => { dx = x; dy = y; }
    }
    if dx != 7 { return 4; }
    if dy != 8 { return 5; }

    return 0;
}
