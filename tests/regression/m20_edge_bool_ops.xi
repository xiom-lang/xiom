// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var t = true; var f = false; if t == f { return 1; } if t != true { return 2; } if !t { return 3; } if f { return 4; } return 0; }