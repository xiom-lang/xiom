// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m175 lock (playground C24, read_line): `xiom_stdin()` returns the C FILE*
// as an Int handle and feeds it to `fgets(..., stream: *UInt8)`. The arg
// coercion used to materialize a pointee-width temporary (truncating the
// FILE* to a byte) and pass its ADDRESS -- read_line returned "" in run mode
// and glibc aborted with "invalid stdio handle" in compile mode. The handle
// bits must inttoptr straight through.
use xiom.io;

fn main() -> Int {
  let line = io.read_line();
  io.println("got: [" + line + "]");
  return 0;
}
