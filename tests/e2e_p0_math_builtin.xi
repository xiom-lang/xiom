// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// P0-4 E2E Test: Math builtin interception
// Tests that math.shl, math.bit_or, etc. compile to native LLVM instructions.
use xiom.math;

fn main() -> Int {
    // Test math.shl -- should be native shl, not function call
    var a = math.shl(1, 4);  // 1 << 4 = 16
    if a != 16 { return 1; }

    // Test math.bit_or -- native or
    var b = math.bit_or(3, 12);  // 3 | 12 = 15
    if b != 15 { return 2; }

    // Test math.bit_and -- native and
    var c = math.bit_and(7, 5);  // 7 & 5 = 5
    if c != 5 { return 3; }

    // Test math.bit_xor -- native xor
    var d = math.bit_xor(7, 3);  // 7 ^ 3 = 4
    if d != 4 { return 4; }

    // Test math.shr -- native ashr
    var e = math.shr(32, 2);  // 32 >> 2 = 8
    if e != 8 { return 5; }

    // Test math.bit_not -- native xor -1
    var f = math.bit_not(0);  // ~0 = -1
    if f != -1 { return 6; }

    return 0;
}
