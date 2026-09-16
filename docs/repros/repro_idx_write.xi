// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module repro_idx_write

type Sched = {
  key: Vec[UInt8];
  rounds: Int;
}

pub fn make_sched(key: &Vec[UInt8]) -> Sched {
  var expanded = Vec[UInt8].new();
  var i = 0;
  while i < 16 {
    expanded.push(0);
    i = i + 1;
  }
  i = 0;
  while i < key.len() {
    expanded[i] = key[i];   // Vec index WRITE
    i = i + 1;
  }
  return Sched{ key: expanded; rounds: 10; };
}

pub fn check_sched(s: &Sched) -> Int {
  if s.rounds != 10 { return 1; }
  if s.key.len() != 16 { return 2; }
  if s.key[15] != 15 { return 3; }
  return 0;
}
