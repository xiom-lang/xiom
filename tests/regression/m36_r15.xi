// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-R15: SOFT keyword used as identifier -- parser handles it.
// (Reserved words like `fn`/`let` are hard errors since the audit #16
// keyword split; soft keywords such as `spawn` remain valid identifiers --
// stdlib relies on Executor.spawn etc.)
var spawn = 42;

fn main() -> Int { return 0; }
