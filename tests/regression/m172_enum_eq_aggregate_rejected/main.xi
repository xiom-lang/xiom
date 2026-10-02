// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m172 lock (selfhost Phase 2 finding (g)): `==` on an enum whose variants
// carry aggregate payloads must be REJECTED with a diagnostic. It used to
// pass the checker and emit `icmp eq %struct.Vec` (invalid IR, clang error).
enum E {
  A,
  B(v: Vec[UInt8]),
}

fn eq(a: E, b: E) -> Bool {
  return a == b;
}

fn main() -> Int {
  return 0;
}
