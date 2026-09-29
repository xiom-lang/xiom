// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// C22 lock companion (packages shape): a package module living at
// packages/c22-pkg/src/hello.xi, declared c22pkg.hello. The module is only
// reachable when the SCRIPT'S directory reaches the catalog index under
// `xiom run` (the run path compiles a temp copy under %TEMP%/xiom_run).
// Mirrors the playground's stageForRun repro (their module is xiom.hello;
// this lock uses a distinct name so stale temp copies cannot collide).
module c22pkg.hello

pub fn hi() -> Int {
  return 4242;
}
