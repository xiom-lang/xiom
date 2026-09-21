// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K11: Closure chain -- pipe closures composed via direct calls
fn main() -> Int { var f = |x| x + 1; var g = |x| x * 2; if f(g(5)) != 11 { return 1; } return 0; }
