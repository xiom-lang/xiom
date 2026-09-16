// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use stdlib.xiom.string;
use stdlib.xiom.io;
fn main() -> Int { var s = "hi"; if string.str_len(s) != 2 { return 1; } return 0; }