// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// m81: same-leaf fn-REFERENCE resolution, module beta.
// Deliberately different body from alpha.is_even so a wrong cross-module
// bind is observable at runtime (beta.check() must be false).
module beta

pub fn is_even(n: Int) -> Bool { return false; }

fn apply(pred: fn(Int) -> Bool, v: Int) -> Bool { return pred(v); }

pub fn check() -> Bool { return apply(is_even, 3); }
