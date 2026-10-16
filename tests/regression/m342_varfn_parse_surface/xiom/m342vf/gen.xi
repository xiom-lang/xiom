// m342 (REL-20261015-1358-engine, W1.3): a reserved-keyword variable name
// (`var fn`) is a RECOVERABLE parse error -- the parser drops the enclosing
// declaration and keeps going. In an IMPORTED module this used to be silent:
// the module loaded into the catalog with the dropped item missing, and the
// importing entry failed with a downstream T001 undefined variable 'calc'.
// The entry fixture main.xi pins that the module's P001 now surfaces (module
// path + position) instead of the misleading unresolved-name cascade.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.m342vf.gen;

pub fn calc(n: Int) -> Int {
    var fn = n + 1;
    return fn * 2;
}
