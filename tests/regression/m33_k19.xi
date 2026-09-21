// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K19: Anonymous fn literal -- inline block closure call
fn main() -> Int { var r = (fn(x: Int) -> Int { return x * 7; })(6); if r != 42 { return 1; } return 0; }
