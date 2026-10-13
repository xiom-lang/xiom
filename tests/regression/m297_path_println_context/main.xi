// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m297 lock (REL-20261010-1615-stdlib, witness tools/known_failures/
// p_path_println_context_crash.xi): `Path.is_dir` bare-calls the same-leaf
// free fn (`is_dir(self.inner)`); the G-10 implicit-self override re-bound
// the already-resolved `io.is_dir` to the sibling method `Path.is_dir`
// itself -- infinite recursion to the depth-500 llvm.trap, exit 0xC000001D
// before any output. Context shape: io import + trailing printlns.
module m297_path_println_context.main

use xiom.path as path;
use xiom.io as io;

fn main() -> Int {
  let pe = path.Path.new("");
  let pex = pe.exists();
  if pex != false { return 1; }
  io.println("exists-ok");
  let pid = pe.is_dir();
  if pid != false { return 2; }
  io.println("is_dir-ok");
  let dot = path.Path.new(".");
  let dot_d = dot.is_dir();
  if dot_d != true { return 3; }
  io.println("dot-dir-ok");
  return 0;
}
