// m342 sibling entry: xiom.m342vf.misc uses `var let` (another reserved
// keyword) -- same recoverable-parse class, same expected surface (P001 with
// the module path + position, no downstream undefined-variable cascade).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m342_varfn_parse_surface.main_reserved;

use xiom.m342vf.misc;
use xiom.io;

fn main() -> Int {
    println(int_to_string(calc_let(4)));
    return 0;
}
