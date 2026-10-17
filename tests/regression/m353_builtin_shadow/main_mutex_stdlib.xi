// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m353 lock (REL-20261011-1456-stdlib), stdlib half: the documented public
// `xiom.sync.Mutex[T]` surface must be reachable through module
// qualification. Pre-fix the checker's builtin `Mutex.new` (0 args) won the
// bare `Mutex.new` lookup: `sync.Mutex.new(42)` failed T001 and the other
// methods were unreachable. Requires the stdlib checkout (xiom/sync/mutex.xi);
// the e2e test skips loudly without it.
module m353_builtin_shadow.main_mutex_stdlib;

use xiom.io;
use xiom.sync;

pub fn main() -> Int {
  var fail = 0;
  let m = sync.Mutex.new(42);
  io.println("made");
  let g = m.lock();
  if g.get() != 42 { fail = fail + 1; }
  g.drop();
  let out = m.into_inner();
  io.println("out=" + out.to_str());
  if out != 42 { fail = fail + 2; }
  io.println("m353 stdlib mutex fail_bits=" + fail.to_str());
  return fail;
}
