// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module test_lib

pub type Result2 = {
  message: Str;
  code: Int;
} derive[Clone, Eq]

pub fn Result2.new(msg: Str, code: Int) -> Result2 {
  return Result2{ message: msg, code: code };
}

pub fn make_greeting(name: Str) -> Str {
  return name;
}
