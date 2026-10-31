// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Module A: scalar `g`. Module B below declares the same global name with a
// struct type. They must get separate LLVM symbols; sharing `@g` made B read
// A's i64 as a `%struct.Box*` (REL-20261030-2357-engine-2).

module xiom.coll.a

pub var g: Int = 1;

pub fn get_a() -> Int { return g; }
pub fn set_a(v: Int) -> Int { g = v; return g; }
