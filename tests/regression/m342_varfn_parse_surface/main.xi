// m342 (REL-20261015-1358-engine, W1.3): the imported module
// xiom.m342vf.gen carries a recoverable parse error (`var fn` -- `fn` is a
// reserved keyword). Pre-fix this compile reported ONLY
// `error[T001]: undefined variable 'calc'` (the dropped declaration
// masqueraded as a name-resolution failure) and exit code 1. Post-fix it
// must report the module's P001 (module path + position) as the failure.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m342_varfn_parse_surface.main;

use xiom.m342vf.gen;
use xiom.io;

fn main() -> Int {
    println(int_to_string(calc(3)));
    return 0;
}
