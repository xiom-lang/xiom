// m42_round12_str_closures — round-12 (2026-08-21) regression:
// Str-returning closures through Result/Err construction — the closure
// thunk's params are i64 (uniform env-first ABI) but their DECLARED XIOM
// types were not tracked, so a Str param used inside the body degraded to
// a scalar: `str_concat("ERR_", e)` truncated the string HANDLE to a byte
// (alloca i8 + trunc i64) — corrupted map_err payloads (garbage bytes).
// Also covers the mono-site fn_local_returns substitution (the generic
// `fn(E) -> F` ret must resolve through type_map so struct returns stay
// BY VALUE) and &T closure params (filter predicates deref through them).
module m42_round12_str_closures
use xiom.core;
use xiom.cmp;
use xiom.string;

fn main() -> Int {
  // 1. map_err with a Str-returning closure (the rm1 shape).
  let err: Result[Int, Str] = Err("oops");
  let mapped_err = err.map_err(fn(e: Str) -> Str { return string.str_concat("ERR_", e); });
  match mapped_err {
    Ok(_) => { return 1; }
    Err(e) => { if e != "ERR_oops" { return 2; } }
  }
  // 2. map_err passthrough through a second map_err (nested Str closures).
  let err2: Result[Int, Str] = Err("bad");
  let nested = err2.map_err(fn(e: Str) -> Str { return string.str_concat("wrapped: ", e); })
                   .map_err(fn(e: Str) -> Str { return string.str_concat("[", e); });
  match nested {
    Ok(_) => { return 3; }
    Err(e) => { if e != "[wrapped: bad" { return 4; } }
  }
  // 3. Str-returning closure consumed by an Int-returning chain (regression
  // guard: the Int path must stay unaffected).
  let ok: Result[Int, Str] = Ok(10);
  let mapped = ok.map(fn(x: Int) -> Int { return x * 2; });
  match mapped {
    Ok(v) => { if v != 20 { return 5; } }
    Err(_) => { return 6; }
  }
  // 4. &T closure params (Option.filter predicate derefs the ref).
  let f = Some(5);
  let filt = f.filter(fn(x: &Int) -> Bool { return *x > 3; });
  if filt.is_none { return 7; }
  let filt2 = f.filter(fn(x: &Int) -> Bool { return *x > 10; });
  if filt2.is_some { return 8; }
  // 5. zero-arg closure returning an enum (Ordering) — struct return via
  // the M20-A1 env-first call with no args. then_with(Equal) calls f();
  // then_with(non-Equal) returns self. Enum-variant receivers must pass
  // the receiver VALUE (previously dropped → undefined self at -O2).
  if cmp.Equal.then_with(fn() -> cmp.Ordering { return cmp.Equal; }) != cmp.Equal { return 9; }
  if cmp.Less.then_with(fn() -> cmp.Ordering { return cmp.Equal; }) != cmp.Less { return 10; }
  if cmp.Greater.reverse() != cmp.Less { return 11; }
  return 0;
}
