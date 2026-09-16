// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M9.6: impl Trait return types -- E2E test
// Verifies that functions with `impl Trait` return types compile to valid LLVM IR.

interface Display {
    fn show() -> Str;
}

fn get_value() -> impl Display {
    return 42;
}

fn main() -> Int {
    return 0;
}
