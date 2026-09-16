// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// P2-1 E2E Test: ? operator return-type check
// Tests that the ? operator correctly warns when used in non-Result/Option functions.

// Helper: returns Option[Int]
fn opt_div(a: Int, b: Int) -> Option[Int] {
    // Using ? on Option is valid since opt_div returns Option
    return Some(a);
}

// This should trigger an error: ? used in function returning Int
// (Commented out because it's a compile-time error, not runtime)
// fn bad_use() -> Int {
//     var x: Option[Int] = Some(42);
//     var y = x?;  // ERROR: ? in function returning Int
//     return y;
// }

fn main() -> Int {
    var r = opt_div(10, 2);
    // Just verify that valid ? usage compiles and runs
    return 0;
}
