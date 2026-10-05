// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m195 real-world lock: `xiom.reflect.all_types()` builds Vec<TypeInfo> with
// angle-bracket type args (128-byte elements) and Vec<FieldInfo> (24-byte).
// Pre-fix the parser dropped <TypeInfo>/<FieldInfo>, both Vecs allocated
// 16 x 8-byte slots and the pushes heap-corrupted: 0xC0000374 after ~8s.
use xiom.reflect;

fn main() -> Int {
  let all = reflect.all_types();
  if all.len() < 0 {
    return 2;
  }
  return 0;
}
