// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var a = true && false; if a { return 1; } var b = true || false; if !b { return 2; } var c = false || (true && true); if !c { return 3; } return 0; }