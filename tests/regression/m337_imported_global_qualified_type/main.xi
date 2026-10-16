// m337 (XVC-C-17, REL-20261015-2216-xvector): an imported-type module-level
// `var` silently misregistered as i64 -- the declared spelling was
// module-qualified (`mutex.SyncMutex` / `xiom.m337sync.mutex.SyncMutex`)
// while the registry key is the bare leaf. Whole-struct stores kept only
// field 0 and every field read returned 0/null (guarded locks became
// unguarded). This fixture locks the qualified-alias + full-path spellings,
// whole-struct assignment (literal + cross-module constructor), reading back
// every field, the INITIAL-value read, and the same-leaf collision variant
// (XVC-C-14 class: ga.Widget{alpha} vs gb.Widget{beta;gamma}) where the
// qualified spelling must bind its own owner. Every check has a distinct
// exit code (10+); 0 = all green. Pre-fix: 2 FAIL / rc 1 on the xvector
// probe; this fixture pins the codegen shapes hermetically.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m337_imported_global_qualified_type.main;

use xiom.m337sync.mutex;
use xiom.m337sync.ga;
use xiom.m337sync.gb;

// A: qualified ALIAS spelling, whole-struct literal store.
var GA: mutex.SyncMutex = mutex.SyncMutex{ locked: null };
// B: qualified alias spelling, cross-module constructor store.
var GB: mutex.SyncMutex = mutex.SyncMutex{ locked: null };
// C: FULL-PATH spelling.
var GC: xiom.m337sync.mutex.SyncMutex = xiom.m337sync.mutex.SyncMutex{ locked: null };
// D: qualified alias spelling with a NON-NULL initial value (runtime init);
// the initial value must be readable before any assignment.
var GD: mutex.SyncMutex = mutex.SyncMutex{ locked: hptr() };
// E: multi-field imported struct -- fields beyond field 0 must survive the
// whole-struct store (the old path kept field 0 only).
var GE: mutex.Guarded = mutex.Guarded{ p: hptr(); v: 3; w: 4 };
// F: same-leaf collision variant (XVC-C-14 class).
var GW: gb.Widget = gb.Widget{ beta: 3; gamma: 4 };
var GV: ga.Widget = ga.Widget{ alpha: 1 };

fn hptr() -> *Int {
  var p: *Int = null;
  unsafe { p = 64 as *Int; }   // never dereferenced; only tested for null
  return p;
}

fn a_ok() -> Bool { return GA.locked != null; }
fn b_ok() -> Bool { return GB.locked != null; }
fn c_ok() -> Bool { return GC.locked != null; }
fn d_ok() -> Bool { return GD.locked != null; }
fn ge_p_ok() -> Bool { return GE.p != null; }
fn ge_v() -> Int { return GE.v; }
fn ge_w() -> Int { return GE.w; }
fn gw_beta() -> Int { return GW.beta; }
fn gw_gamma() -> Int { return GW.gamma; }
fn gv_alpha() -> Int { return GV.alpha; }

fn main() -> Int {
  // D: the initial value of a runtime-initialized global is visible first.
  if !d_ok() { return 13; }

  // A: absolute const pointer through the qualified alias spelling.
  if GA.locked != null { return 9; }   // initial null
  GA = mutex.SyncMutex{ locked: hptr() };
  if !a_ok() { return 10; }

  // B: cross-module constructor returning the imported struct.
  GB = mutex.mutex_new();
  if !b_ok() { return 11; }

  // C: full-path spelling.
  GC = xiom.m337sync.mutex.SyncMutex{ locked: hptr() };
  if !c_ok() { return 12; }

  // E: every field of the multi-field struct, initial and after a store.
  if !ge_p_ok() { return 14; }
  if ge_v() != 3 { return 15; }
  if ge_w() != 4 { return 16; }
  GE = mutex.guarded_new();
  if !ge_p_ok() { return 17; }
  if ge_v() != 11 { return 18; }
  if ge_w() != 22 { return 19; }

  // F: same-leaf collision -- each qualified global keeps its own layout.
  if gv_alpha() != 1 { return 20; }
  if gw_beta() != 3 { return 21; }
  if gw_gamma() != 4 { return 22; }
  GV = ga.Widget{ alpha: 7 };
  if gv_alpha() != 7 { return 23; }
  GW = gb.Widget{ beta: 8; gamma: 9 };
  if gw_beta() != 8 { return 24; }
  if gw_gamma() != 9 { return 25; }

  return 0;
}
