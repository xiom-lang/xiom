// m350 (REL-20261016-2016-packages): reading `r.value` / `r.error` behind a
// `&Result[_, _]` (or `&Option[_]`) parameter CRASHED 0xC0000005 when the
// payload is a user struct/enum. `param_llvm_type`'s Ref arm lowers every
// `&Result[..]`/`&Option[..]` to the BASE erased `%struct.Result*`/
// `%struct.Option*` (i64 handle slots), but every VALUE of a struct/enum-payload
// container is materialized with the concrete monomorph
// (`ensure_concrete_result`/`ensure_concrete_option`) whose payload is INLINE
// (%struct.Result__P__Str = { i64, %struct.P, i8* }); the caller bitcasts
// monomorph -> base, and the pre-fix pointer-to-struct field reader
// (expr.rs, GEP on the base pointee + `field_llvm_type` i64) read field 1 as an
// i64 HANDLE (the payload's first word / .error at base offset 16 mid-payload),
// so the i64->struct coercion inttoptr'd garbage -> AV (.value) or the Str
// reader strlen'd garbage -> len 0xFFFFFFFF (.error). m345 fixed only the
// clause REBIND locals; this reader site was untouched.
//
// Arms: A struct payload `.value`; B mut-ref variant; C the jev-exact shape
// (`&Result[Q, ScratchErr]`: `.error` match + `.value`); D `&Option[P]`
// `.value`; E `&Result[Str, ScratchErr]` `.error` bound to a local + match.
// Controls (base layout / by-value / locals; green in BOTH states): F Int
// payload, G Vec payload, H Str payload, I by-value param, J local `.value`,
// K enum-err match on a scalar Ok payload (offset-accident layout), L/M
// is_ok/is_err/is_some on ref params, N Err-path vacuity.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m350_result_ref_field_read;

use xiom.io;
use xiom.convert;

pub enum ScratchErr {
  BadRequest(reason: Str),
  HttpStatus(code: Int, detail: Str),
}

pub type P = {
  a: Int;
  b: Int;
}

pub type Q = {
  kind: Int;
  id: Str;
  labels: Vec[Str];
}

fn ok_p(a: Int, b: Int) -> Result[P, Str] {
  return Ok(P{ a: a, b: b });
}

fn ok_q(v: Q) -> Result[Q, ScratchErr] {
  return Ok(v);
}

fn err_q(e: ScratchErr) -> Result[Q, ScratchErr] {
  return Err(e);
}

fn err_str(e: Str) -> Result[Str, ScratchErr] {
  return Err(ScratchErr.BadRequest(e));
}

fn some_p(a: Int, b: Int) -> Option[P] {
  return Some(P{ a: a, b: b });
}

fn ok_int(v: Int) -> Result[Int, Int] {
  return Ok(v);
}

fn ok_vec(v: Vec[Int]) -> Result[Vec[Int], Str] {
  return Ok(v);
}

fn ok_s(s: Str) -> Result[Str, Str] {
  return Ok(s);
}

fn err_scalar(e: ScratchErr) -> Result[Int, ScratchErr] {
  return Err(e);
}

// Arm A: struct payload `.value` behind `&Result[P, Str]` (red pre-fix: AV).
fn get_a(r: &Result[P, Str]) -> Int {
  if !r.is_ok { return -1; }
  let p: P = r.value;
  return p.a;
}

// Arm B: the `&mut` twin -- same base-erased pointer ABI.
fn get_a_mut(r: &mut Result[P, Str]) -> Int {
  if !r.is_ok { return -1; }
  let p: P = r.value;
  return p.b;
}

// Arm C: the jev-lane shape -- enum error behind `&Result[Q, ScratchErr]`,
// `.error` matched and `.value` read (red pre-fix: AV on both paths).
fn bump(r: &Result[Q, ScratchErr]) -> Int {
  if !r.is_ok {
    match r.error {
      ScratchErr.BadRequest(reason) => { return 1; },
      ScratchErr.HttpStatus(code, detail) => { return code; },
    };
  }
  let q: Q = r.value;
  return q.labels.len();
}

// Arm D: `&Option[P]` `.value` (red pre-fix: AV).
fn opt_a(o: &Option[P]) -> Int {
  if !o.is_some { return -1; }
  let p: P = o.value;
  return p.a;
}

// Arm E: enum error behind `&Result[Str, ScratchErr]`, bound to a local and
// matched (red pre-fix: 0xC000001D on the misread enum).
fn str_err_len(r: &Result[Str, ScratchErr]) -> Int {
  if r.is_ok { return -1; }
  let e: ScratchErr = r.error;
  match e {
    ScratchErr.BadRequest(reason) => { return reason.len(); },
    ScratchErr.HttpStatus(code, detail) => { return code; },
  };
}

// Control F: Int payload stays in the base %struct.Result layout.
fn int_val(r: &Result[Int, Int]) -> Int {
  if !r.is_ok { return -1; }
  let v: Int = r.value;
  return v;
}

// Control G: Vec payload stays on the base handle layout.
fn vec_len(r: &Result[Vec[Int], Str]) -> Int {
  if !r.is_ok { return -1; }
  let v: Vec[Int] = r.value;
  return v.len();
}

// Control H: Str payload stays on the base handle layout.
fn str_len(r: &Result[Str, Str]) -> Int {
  if !r.is_ok { return -1; }
  let s: Str = r.value;
  return s.len();
}

// Control I: by-value struct payload param (already lowered concretely).
fn byval_a(r: Result[P, Str]) -> Int {
  if !r.is_ok { return -1; }
  let p: P = r.value;
  return p.a;
}

// Control K: enum error behind `&Result[Int, ScratchErr]` -- the scalar Ok
// payload keeps the inline error at the historical base offset 16.
fn err_scalar_code(r: &Result[Int, ScratchErr]) -> Int {
  if r.is_ok { return -1; }
  match r.error {
    ScratchErr.BadRequest(reason) => { return 1; },
    ScratchErr.HttpStatus(code, detail) => { return code; },
  };
}

// Controls L/M: tag-only predicates on ref params (offset 0 in both layouts).
fn is_ok_of(r: &Result[P, Str]) -> Int {
  if r.is_ok { return 1; }
  return 0;
}

fn is_some_of(o: &Option[P]) -> Int {
  if o.is_some { return 1; }
  return 0;
}

fn is_err_of(r: &Result[Str, ScratchErr]) -> Int {
  if r.is_err { return 1; }
  return 0;
}

pub fn main() -> Int {
  var fails: Int = 0;

  // A: struct payload .value
  let ra: Result[P, Str] = ok_p(7, 9);
  let na = get_a(&ra);
  if na != 7 {
    fails = fails + 1;
    io.println("m350 [A]: got " + convert.int_to_string(na));
  }

  // B: &mut twin
  var rb: Result[P, Str] = ok_p(3, 4);
  let nb = get_a_mut(&mut rb);
  if nb != 4 {
    fails = fails + 1;
    io.println("m350 [B]: got " + convert.int_to_string(nb));
  }

  // C: jev-exact enum error + value
  var q: Q = Q{ kind: 1, id: "q1", labels: Vec[Str].new() };
  q.labels.push("alpha");
  q.labels.push("beta");
  let rc_ok: Result[Q, ScratchErr] = ok_q(q);
  let nc = bump(&rc_ok);
  if nc != 2 {
    fails = fails + 1;
    io.println("m350 [C-ok]: got " + convert.int_to_string(nc));
  }
  let rc_err: Result[Q, ScratchErr] = err_q(ScratchErr.BadRequest("no"));
  let nc2 = bump(&rc_err);
  if nc2 != 1 {
    fails = fails + 1;
    io.println("m350 [C-err]: got " + convert.int_to_string(nc2));
  }

  // D: &Option[P]
  let rd: Option[P] = some_p(11, 12);
  let nd = opt_a(&rd);
  if nd != 11 {
    fails = fails + 1;
    io.println("m350 [D]: got " + convert.int_to_string(nd));
  }

  // E: Str ok-payload with enum error
  let re: Result[Str, ScratchErr] = err_str("no");
  let ne = str_err_len(&re);
  if ne != 2 {
    fails = fails + 1;
    io.println("m350 [E]: got " + convert.int_to_string(ne));
  }

  // F: Int payload control
  let rf: Result[Int, Int] = ok_int(42);
  let nf = int_val(&rf);
  if nf != 42 {
    fails = fails + 1;
    io.println("m350 [F]: got " + convert.int_to_string(nf));
  }

  // G: Vec payload control
  var v: Vec[Int] = Vec[Int].new();
  v.push(5);
  v.push(7);
  let rg: Result[Vec[Int], Str] = ok_vec(v);
  let ng = vec_len(&rg);
  if ng != 2 {
    fails = fails + 1;
    io.println("m350 [G]: got " + convert.int_to_string(ng));
  }

  // H: Str payload control
  let rh: Result[Str, Str] = ok_s("hello");
  let nh = str_len(&rh);
  if nh != 5 {
    fails = fails + 1;
    io.println("m350 [H]: got " + convert.int_to_string(nh));
  }

  // I: by-value param control
  let ri: Result[P, Str] = ok_p(13, 14);
  let ni = byval_a(ri);
  if ni != 13 {
    fails = fails + 1;
    io.println("m350 [I]: got " + convert.int_to_string(ni));
  }

  // J: local .value control
  let rj: Result[P, Str] = ok_p(21, 22);
  if !rj.is_ok {
    fails = fails + 1;
    io.println("m350 [J]: not ok");
  } else {
    let pj: P = rj.value;
    if pj.a != 21 {
      fails = fails + 1;
      io.println("m350 [J]: got " + convert.int_to_string(pj.a));
    }
  }

  // K: enum-err match on scalar payload control
  let rk: Result[Int, ScratchErr] = err_scalar(ScratchErr.BadRequest("no"));
  let nk = err_scalar_code(&rk);
  if nk != 1 {
    fails = fails + 1;
    io.println("m350 [K]: got " + convert.int_to_string(nk));
  }

  // L/M: tag-only predicates
  let rl: Result[P, Str] = ok_p(1, 1);
  if is_ok_of(&rl) != 1 {
    fails = fails + 1;
    io.println("m350 [L1]: is_ok");
  }
  if is_some_of(&rd) != 1 {
    fails = fails + 1;
    io.println("m350 [L2]: is_some");
  }
  if is_err_of(&re) != 1 {
    fails = fails + 1;
    io.println("m350 [L3]: is_err");
  }

  // N: the Err path of arm A stays vacuous.
  let rn: Result[Str, ScratchErr] = err_str("nope");
  let nn = str_err_len(&rn);
  if nn != 4 {
    fails = fails + 1;
    io.println("m350 [N]: got " + convert.int_to_string(nn));
  }

  io.println("m350_result_ref_field_read: " + convert.int_to_string(fails) + " failing checks");
  return fails;
}
