// REL-20261022-2032 lock (playground relay): a WARM script cache must
// stream interactive output. The cached path used Command::output(), so the
// prompt only appeared after stdin closed (and stderr was dropped, and a
// non-zero exit re-ran the program).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module c25_run_cached_live

use xiom.io;

fn main() -> Int {
  io.println("Name?");
  let name = io.read_line();
  io.println("Hello, " + name + "!");
  return 0;
}
