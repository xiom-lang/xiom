// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// Same module LEAF (base32) and same fn names as alpha.base32, delegating
// through an alias (`canon.*`). R15b: the emitter must bind these calls to
// alpha's qualified symbol; pre-fix it bound the caller's own symbol.
module beta.base32
use alpha.base32 as canon;

pub fn encode(x: Int) -> Int { return canon.encode(x); }
pub fn name() -> Str { return canon.name(); }
