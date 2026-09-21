// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module cg02_verify
var x: Float32 = 0.5;
var y: Float32 = 0.3;
fn main() -> Int {
    if x > 0.4 { return 0; }
    return 1;
}
