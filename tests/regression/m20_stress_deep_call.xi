// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn a(x: Int) -> Int { return x + 1; }
fn b(x: Int) -> Int { return a(x) + 1; }
fn c(x: Int) -> Int { return b(x) + 1; }
fn d(x: Int) -> Int { return c(x) + 1; }
fn e(x: Int) -> Int { return d(x) + 1; }
fn main() -> Int { if e(0) != 5 { return 1; } return 0; }