// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E06: 25-variant enum -- many unit variants, match exhaustive
enum ManyE { V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12, V13, V14, V15, V16, V17, V18, V19, V20, V21, V22, V23, V24, V25 }
fn kind_str(t: ManyE) -> Int {
  match t {
    V1 => { return 1; }
    V2 => { return 2; }
    V3 => { return 3; }
    V4 => { return 4; }
    V5 => { return 5; }
    V6 => { return 6; }
    V7 => { return 7; }
    V8 => { return 8; }
    V9 => { return 9; }
    V10 => { return 10; }
    V11 => { return 11; }
    V12 => { return 12; }
    V13 => { return 13; }
    V14 => { return 14; }
    V15 => { return 15; }
    V16 => { return 16; }
    V17 => { return 17; }
    V18 => { return 18; }
    V19 => { return 19; }
    V20 => { return 20; }
    V21 => { return 21; }
    V22 => { return 22; }
    V23 => { return 23; }
    V24 => { return 24; }
    V25 => { return 25; }
  }
}
fn main() -> Int {
  if kind_str(ManyE.V1) != 1 { return 1; }
  if kind_str(ManyE.V2) != 2 { return 2; }
  if kind_str(ManyE.V3) != 3 { return 3; }
  if kind_str(ManyE.V4) != 4 { return 4; }
  if kind_str(ManyE.V5) != 5 { return 5; }
  if kind_str(ManyE.V6) != 6 { return 6; }
  if kind_str(ManyE.V7) != 7 { return 7; }
  if kind_str(ManyE.V8) != 8 { return 8; }
  if kind_str(ManyE.V9) != 9 { return 9; }
  if kind_str(ManyE.V10) != 10 { return 10; }
  if kind_str(ManyE.V11) != 11 { return 11; }
  if kind_str(ManyE.V12) != 12 { return 12; }
  if kind_str(ManyE.V13) != 13 { return 13; }
  if kind_str(ManyE.V14) != 14 { return 14; }
  if kind_str(ManyE.V15) != 15 { return 15; }
  if kind_str(ManyE.V16) != 16 { return 16; }
  if kind_str(ManyE.V17) != 17 { return 17; }
  if kind_str(ManyE.V18) != 18 { return 18; }
  if kind_str(ManyE.V19) != 19 { return 19; }
  if kind_str(ManyE.V20) != 20 { return 20; }
  if kind_str(ManyE.V21) != 21 { return 21; }
  if kind_str(ManyE.V22) != 22 { return 22; }
  if kind_str(ManyE.V23) != 23 { return 23; }
  if kind_str(ManyE.V24) != 24 { return 24; }
  if kind_str(ManyE.V25) != 25 { return 25; }
  return 0;
}