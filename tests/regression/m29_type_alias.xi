// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M29: Type alias chain and const
type Id = Int;
type UserId = Id;
type SessionId = UserId;
const MAGIC: SessionId = 42;
fn get() -> SessionId { return MAGIC; }
fn main() -> Int {
  var s: SessionId = get();
  if s == 42 { return 0; }
  return 1;
}
