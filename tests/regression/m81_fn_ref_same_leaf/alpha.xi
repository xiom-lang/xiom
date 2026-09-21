// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m81: same-leaf fn-REFERENCE resolution, module alpha.
// `is_even` exists in alpha, beta and (catalog) bodies; the fn VALUE passed
// to `apply` must bind alpha's own `is_even` (true), never beta's (false).
// Pre-R25 the suffix scan over the fn registry was HashMap-ordered.
module alpha

pub fn is_even(n: Int) -> Bool { return true; }

fn apply(pred: fn(Int) -> Bool, v: Int) -> Bool { return pred(v); }

pub fn check() -> Bool { return apply(is_even, 3); }
