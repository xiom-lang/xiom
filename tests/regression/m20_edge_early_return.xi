// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn find(arr: Vec[Int], len: Int, target: Int) -> Int { var i = 0; while i < len { if arr[i] == target { return i; } i = i + 1; } return -1; }
fn main() -> Int { if find(Vec[Int]{}, 0, 5) != -1 { return 1; } return 0; }