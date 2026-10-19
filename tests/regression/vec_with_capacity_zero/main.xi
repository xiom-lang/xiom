// Zero-capacity Vec buffer regression (REL-20261019-1624-stdlib):
// `Vec[UInt8]::with_capacity(0)` is a valid empty buffer (io.read_file on an
// empty file). The guard allocator returns NULL for size <= 0; the Vec ctor
// must not trap on that valid zero-capacity state. A later push must grow the
// buffer safely.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module vec_with_capacity_zero

use xiom.io;

fn main() -> Int {
  var v: Vec[UInt8] = Vec[UInt8]::with_capacity(0);
  if v.len() != 0 { return 1; }
  v.push(7);
  if v.len() != 1 { return 2; }
  if v[0] != 7 { return 3; }
  io.println("ok");
  return 0;
}
