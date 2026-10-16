// m337 (XVC-C-17, REL-20261015-2216-xvector): the reporter's REAL shape --
// imported STDLIB type (`use xiom.sync.mutex;`) in module-level `var`
// globals. Pre-fix the SyncMutex slot was emitted as i64 (5x "unknown type
// 'mutex.SyncMutex' -- defaulting to i64" warnings): whole-struct stores
// kept field 0 bits only and every read returned null, so the G-1 global
// lock pattern was silently unguarded. Requires the stdlib checkout
// (xiom/sync/mutex.xi); distinct exit codes 9+; 0 = green.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m337_imported_global_qualified_type.main_stdlib;

use xiom.sync.mutex;

// Qualified ALIAS spelling; the registry key for the stdlib type is the
// bare leaf `SyncMutex`.
var G_M: mutex.SyncMutex = mutex.SyncMutex{ locked: null };
// FULL-PATH spelling of the same imported type.
var G_F: xiom.sync.mutex.SyncMutex = xiom.sync.mutex.SyncMutex{ locked: null };

fn hptr() -> *Int {
  var p: *Int = null;
  unsafe { p = 64 as *Int; }   // never dereferenced; only tested for null
  return p;
}

fn g_m_ok() -> Bool { return G_M.locked != null; }
fn g_f_ok() -> Bool { return G_F.locked != null; }

fn main() -> Int {
  if G_M.locked != null { return 9; }     // initial null
  G_M = mutex.mutex_new();                // cross-module constructor
  if !g_m_ok() { return 10; }

  G_M = mutex.SyncMutex{ locked: hptr() };  // whole-struct literal store
  if !g_m_ok() { return 11; }

  G_F = xiom.sync.mutex.mutex_new();
  if !g_f_ok() { return 12; }

  return 0;
}
