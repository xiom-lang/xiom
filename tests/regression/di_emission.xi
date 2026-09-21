// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// E2E: DI emission -- compile with --debug and verify function names in binary
fn add(a: Int, b: Int) -> Int { return a + b; }
fn mul(a: Int, b: Int) -> Int { return a * b; }
fn main() -> Int { return add(mul(3, 4), 5); }
