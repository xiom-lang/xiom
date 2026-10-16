// m337 (XVC-C-17, REL-20261015-2216-xvector): hermetic stand-in for
// xiom.sync.mutex -- an IMPORTED module whose struct type is consumed by
// module-level `var` globals in the entry file. `SyncMutex.locked` is a
// pointer field whose pre-fix global slot was emitted as i64 (only field 0
// bits reached the slot; every read returned null). `Guarded` has fields
// beyond field 0 (v/w) which the old store path discarded entirely.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.m337sync.mutex;

pub type SyncMutex = { locked: *Int; }

pub fn mutex_new() -> SyncMutex {
  var p: *Int = null;
  unsafe { p = 64 as *Int; }
  return SyncMutex{ locked: p; };
}

pub type Guarded = { p: *Int; v: Int; w: Int; }

pub fn guarded_new() -> Guarded {
  var p: *Int = null;
  unsafe { p = 96 as *Int; }
  return Guarded{ p: p; v: 11; w: 22; };
}
