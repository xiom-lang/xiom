// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int {
  // M18: Char match basic -- should work
  var r1 = match Some('a') { Some('a') => 1; _ => 0; };
  if r1 != 1 { return 1; }

  // M18: Char match with variable  
  var c: Char = 'a';
  var r2 = match Some(c) { Some('a') => 1; _ => 0; };
  if r2 != 1 { return 2; }

  // M18: None match
  var r3 = match None { Some('a') => 1; _ => 0; };
  if r3 != 0 { return 3; }

  // NOTE: OR-pattern Some('a')|Some('b') deferred -- known M18 limitation
  
  return 0;
}
