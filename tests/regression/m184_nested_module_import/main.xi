// m184 lock (packages relay): `module pkg.tests` + `use pkg;` must see the
// peer pkg.xi root exports. Before the fix the program's own nested chain
// registered a leaf-less `pkg` entry, the file-backed module was never
// loaded, and every root export was missing (163 T001s in the packages'
// websocket).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module pkg.tests

use pkg;

fn main() -> Int {
  if pkg.val() != 7 { return 1; }
  return 0;
}
