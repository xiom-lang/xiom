// m2057 (REL-20261019-2057-engine): named fn-type alias parameters were
// typed as Unit in call position, so `f(x)` checked as a Unit-returning call.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Covers the three engine shapes: a same-module alias, a module-qualified
// alias, and a fn-typed STRUCT FIELD (literal + reassignment). Also locks
// qualified numeric aliases in arithmetic (`Count + 1`).
//
// Pre-fix the checker left the alias as a Named type and the fn-call path
// returned Unit; codegen then lowered an alias param as a raw function
// pointer instead of the i64 env ABI, and a field call fell through to an
// unresolved `Job.cb` symbol. The single-letter `type F` repro was masked by
// the generic-param fallback, so these names are deliberately multi-letter.

module m2057_alias_params

use xiom.io;
use xiom.convert;
use m2057_job_types;

type JobFn = fn(Int) -> Int;

type Job = { cb: JobFn; }

fn inc(x: Int) -> Int { return x + 1; }

fn dec(x: Int) -> Int { return x - 1; }

fn inc_count(x: m2057_job_types.Count) -> m2057_job_types.Count { return x + 1; }

fn call_bare(f: JobFn, x: Int) -> Int { return f(x); }

fn call_qualified(f: m2057_job_types.JobFn, x: Int) -> Int { return f(x); }

fn call_field(j: Job, x: Int) -> Int { return j.cb(x); }

fn main() -> Int {
  let a = call_bare(inc, 41);
  let b = call_qualified(inc, 41);
  let j = Job{ cb: inc };
  let c = call_field(j, 41);
  var k = Job{ cb: inc };
  k.cb = dec;
  let d = call_field(k, 42);
  let e = inc_count(41);
  if a != 42 { return 1; }
  if b != 42 { return 2; }
  if c != 42 { return 3; }
  if d != 41 { return 4; }
  if e != 42 { return 5; }
  io.println("alias_ok " + convert.int_to_string(a + b + c + d + e));
  return 0;
}
