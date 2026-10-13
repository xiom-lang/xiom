// m282 (REL-20261011-2053-engine + REL-20261011-1627/1402-xvector): a
// fn-typed PARAM cast to a raw pointer must hand over the RAW CODE address,
// not the two-word closure ENV handle. Pre-fix `f as *UInt8` inttoptr'd the
// env pointer, so the C side calling it ran into heap memory (0xC0000005;
// xiom.thread.spawn's runtime trampoline crashed before the worker body ran).
// Host: host.c (call6 takes a raw code pointer, invokes it, returns the
// value). Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m282_fn_env_raw_ptr

extern "C" {
  fn call6(fp: *UInt8) -> Int;
}

fn six() -> Int { return 6; }

fn via_param(f: fn() -> Int) -> Int
  requires: true
{
  unsafe {
    let raw = f as *UInt8;
    return call6(raw);
  }
}

fn via_local(f: fn() -> Int) -> Int
  requires: true
{
  unsafe {
    let g = f;
    let raw = g as *UInt8;
    return call6(raw);
  }
}

fn main() -> Int {
  var failed = 0;
  let a = via_param(six);
  let b = via_local(six);
  if a != 6 { failed = failed + 1; }
  if b != 6 { failed = failed + 1; }
  return failed;
}
