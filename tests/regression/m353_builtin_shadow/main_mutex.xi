// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m353 lock (REL-20261011-1456-stdlib), hermetic half: the builtin Mutex
// barrier must not shadow a module-declared generic `Mutex[T]`. Distinct fail
// bits pin each builtin key: `new` (1), `get` (2), `lock` (4), `unlock` (8),
// and a second instantiation (16).
module m353_builtin_shadow.main_mutex;

use xiom.io;
use xiom.m353x.mutex;

pub fn main() -> Int {
  var fail = 0;
  let m = mutex.Mutex.new(7);
  io.println("get=" + m.get().to_str());
  if m.get() != 7 { fail = fail + 1; }
  if m.lock() != 7 { fail = fail + 2; }
  if !m.unlock() { fail = fail + 4; }
  let s = mutex.Mutex.new("hi");
  io.println("str=" + s.get());
  if s.get() != "hi" { fail = fail + 8; }
  if s.lock() != "hi" { fail = fail + 16; }
  io.println("m353 mutex fail_bits=" + fail.to_str());
  return fail;
}
