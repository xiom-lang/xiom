// C25 lock (playground relay): a WARM script cache must still deliver piped
// stdin to the program. The cached path ran Command::output(), whose
// default stdin is NULL -- cached reruns printed `got: []` while the cold
// run read `Ada`.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module c25_run_cached_stdin

use xiom.io;

fn main() -> Int {
  let name = io.read_line();
  io.println("got: [" + name + "]");
  return 0;
}
