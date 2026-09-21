// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-G11: Generic type alias
type Id = Int;
type UserId = Id;
fn id[T](x: T) -> T { return x; }
fn get_id(x: UserId) -> UserId { return id(x); }
fn main() -> Int {
  var a: UserId = get_id(55);
  if a != 55 { return 1; }
  return 0;
}
