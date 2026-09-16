// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O13: Option is_some -- direct .is_some() method call
fn main() -> Int {
  var a = Some(42);
  var b: Option[Int] = None;
  var c = Some(0);
  if a.is_some() != true { return 1; }
  if b.is_some() != false { return 2; }
  if c.is_some() != true { return 3; }
  var d: Option[Str] = Some("test");
  if d.is_some() != true { return 4; }
  var e: Option[Str] = None;
  if e.is_some() != false { return 5; }
  return 0;
}
