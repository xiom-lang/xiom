// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var t = true; var f = false; if (t && f) == true { return 1; } if (t || f) != true { return 2; } return 0; }