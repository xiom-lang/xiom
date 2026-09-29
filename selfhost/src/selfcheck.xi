// XIOM -- Selfhost Phase 0 helper selfcheck
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Asserts the pure-XIOM runtime_ffi ports against the C helpers' known
// outputs (stdlib/runtime/xiom_runtime.c). Run by the harness:
// `target/selfhost/xiomc-self --selfcheck` must print SELFCHECK OK and exit
// 0 (crates/xiom-codegen/tests/full_diff_tests.rs::runtime_ffi_selfcheck).
//
// Diagnostics go to stdout because the Phase 0 skeleton has no stderr
// writer; the selfcheck runs standalone (--selfcheck), never on the
// IR-emission path.

module selfhost_selfcheck

use xiom.io;
use selfhost_runtime_ffi;

fn expect_int(actual: Int, expected: Int, what: Str) -> Int {
  if actual != expected {
    io.println("SELFCHECK FAIL: " + what);
    return 1;
  }
  return 0;
}

fn expect_true(actual: Bool, what: Str) -> Int {
  if !actual {
    io.println("SELFCHECK FAIL: " + what);
    return 1;
  }
  return 0;
}

fn expect_str(actual: Str, expected: Str, what: Str) -> Int {
  if actual != expected {
    io.println("SELFCHECK FAIL: " + what);
    return 1;
  }
  return 0;
}

pub fn run() -> Int {
  var failures = 0;

  // --- String access (xiom_str_len / xiom_char_at / xiom_str_slice) ------
  failures = failures + expect_int(selfhost_runtime_ffi.rt_str_len("hello"), 5, "str_len(hello)");
  failures = failures + expect_int(selfhost_runtime_ffi.rt_str_len(""), 0, "str_len(empty)");
  failures = failures + expect_int(selfhost_runtime_ffi.rt_char_at("A", 0), 65, "char_at(A,0)");
  failures = failures + expect_int(selfhost_runtime_ffi.rt_char_at("A", -1), 0, "char_at(A,-1)");
  failures = failures + expect_int(selfhost_runtime_ffi.rt_char_at("A", 9), 0, "char_at(A,oob)");
  failures = failures + expect_int(selfhost_runtime_ffi.rt_char_at("\xE9", 0), 233, "char_at(U+00E9)");
  failures = failures + expect_int(selfhost_runtime_ffi.rt_str_len("\xE9"), 2, "str_len(U+00E9)=2");
  failures = failures + expect_str(selfhost_runtime_ffi.rt_str_slice("hello", 1, 3), "el", "str_slice(mid)");
  failures = failures + expect_str(selfhost_runtime_ffi.rt_str_slice("hello", 3, 3), "", "str_slice(empty)");
  failures = failures + expect_str(selfhost_runtime_ffi.rt_str_slice("hello", -5, 99), "hello", "str_slice(clamp)");

  // --- Symbol table (xiom_intern / xiom_lookup) --------------------------
  var syms = selfhost_runtime_ffi.rt_symbol_table_new();
  let id_a = syms.intern("alpha");
  let id_b = syms.intern("beta");
  let id_a2 = syms.intern("alpha");
  failures = failures + expect_int(id_a, 1, "intern(alpha) id=1");
  failures = failures + expect_int(id_b, 2, "intern(beta) id=2");
  failures = failures + expect_int(id_a2, 1, "intern dedupe");
  failures = failures + expect_int(syms.intern(""), 0, "intern(empty)=0");
  failures = failures + expect_int(syms.count(), 2, "symbol count");
  match syms.lookup(2) {
    Some(s) => { failures = failures + expect_str(s, "beta", "lookup(2)=beta"); }
    None => {
      io.println("SELFCHECK FAIL: lookup(2) returned None");
      failures = failures + 1;
    }
  }
  match syms.lookup(0) {
    Some(_) => {
      io.println("SELFCHECK FAIL: lookup(0) returned Some");
      failures = failures + 1;
    }
    None => {}
  }

  // --- Function table (xiom_fn_table_*) ----------------------------------
  var fns = selfhost_runtime_ffi.rt_fn_table_new();
  fns.add(1, 2, 0, 10, 20);
  fns.add(3, 4, 2, 30, 40);
  failures = failures + expect_int(fns.count(), 2, "fn table count");

  // --- IR buffer (ir_open / ir_raw / text) -------------------------------
  var out = selfhost_runtime_ffi.rt_ir_open();
  out.raw("; XIOM");
  out.raw("define i64 @main() {");
  let text = out.text();
  failures = failures + expect_true(text.len() > 0, "ir text non-empty");
  failures = failures + expect_str(out.text(), "; XIOM\ndefine i64 @main() {\n", "ir text content");

  if failures == 0 {
    io.println("SELFCHECK OK");
    return 0;
  }
  io.println("SELFCHECK: failures reported above");
  return 1;
}
