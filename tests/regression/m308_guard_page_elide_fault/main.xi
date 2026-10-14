// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m308 fault-containment control (REL-20261010-1554-benchmark): an
// ARENA-FREE confined block still runs through the SEH/sigsetjmp trampoline
// after the guard-page arm/disarm elision; a hardware fault inside it must
// still be contained (fault code surfaces, no crash). The e2e test runs
// this with XIOM_TRACE_FAULT=1 and asserts the [trampoline] fault line.
module m308_guard_page_elide_fault.main

fn main() -> Int
{
  var p: *Int;
  unsafe { p = 0 as *Int; }
  var x: Int = 42;
  unsafe { x = *p; }
  if x != 0 { return 1; }
  return 0;
}
