// m342 (REL-20261015-1358-engine, W1.3) sibling case: ANY reserved keyword
// as a `var`/`let` name is the same recoverable-parse class as `var fn`
// (parse_ident rejects RESERVED_WORDS wholesale). This module uses `var let`;
// the entry fixture main_reserved.xi must get the module's P001, not a
// downstream undefined-variable cascade.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.m342vf.misc;

pub fn calc_let(n: Int) -> Int {
    var let = n + 1;
    return let * 2;
}
