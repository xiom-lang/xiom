// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m308 lock (REL-20261010-1554-playground): the prompt printed with
// io.println must reach a PIPE reader BEFORE io.read_line() blocks. Pre-fix
// stdout block-buffered when piped, so the prompt arrived only at exit
// (strace: the whole output as one write after EOF). The e2e test spawns
// this binary with piped stdin and waits for "PROMPT>" WITHOUT writing any
// input first.
module m308_stdout_flush.main

use xiom.io;

fn main() -> Int
{
  io.println("PROMPT>");
  let line = io.read_line();
  io.println("GOT:" + line);
  return 0;
}
