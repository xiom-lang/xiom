// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m292 lock (REL-20261010-1548-bindings-4): a NESTED program module whose
// leaf collides with an import segment (`module ...ffi` + `use xiom.ffi;`)
// flattens a bare "ffi" key into the checker's module map; the import
// worklist then skipped the catalog pre-load of xiom.ffi because its guard
// tested the leaf only. The use bound the program's own module and the
// stdlib submodule fns (xiom.ffi.c) never registered -- bare `c_abs` was
// T001 undefined variable (pre-fix binary: exit 1, 6:13 + 7:9 cascade).
// The fix skips catalog pre-load only for single-segment program-local
// modules. `c_abs` stands in for the filed `c_strlen` call to keep the
// lock runnable (strlen would need a raw C pointer).
module m292_ffi_alias_shadow.ffi

use xiom.ffi;

fn main() -> Int {
  let v = c_abs(-7);
  if (v == 7) { return 0; }
  return 1;
}
