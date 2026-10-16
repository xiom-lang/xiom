// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m338 lock (REL-20261016-1128-engine class 1): an enum VARIANT named `Str`
// in this module must not shadow the builtin `Str` qualifier. The m315
// current-module variant branch outranked builtins at the ident-resolution
// step, so `Str::from_utf8(buf)` typed its receiver as the parent enum
// (JLValue), the builtin Str-assoc table was never reached, and the call fell
// to the unknown-method fallback: T001 "cannot call 'from_utf8' on this
// expression" -- the engine's json_lite.xi:196 shape, red at 0.64.5 and at
// the m334 build, green at 0.64.4. Scalar-named variants must not hijack
// builtin assoc calls in the same module; non-scalar variant/type collisions
// (geom's Vec3) keep m315's precedence.
module xiom.m338x.json_lite;

pub type JLValue = enum {
  Null,
  Str(value: Str),
}

pub type JLParsed = { value: JLValue; pos: Int; }

pub fn jl_str(s: Str) -> JLValue { return JLValue.Str(s); }

pub fn jl_probe(pos: Int) -> Result[JLParsed, Str] {
  var buf = Vec[UInt8].new();
  buf.push(65); // 'A'
  return Ok(JLParsed{ value: jl_str(Str::from_utf8(buf)); pos: pos + 1 });
}

pub fn jl_str_of(v: JLValue) -> Str {
  match v {
    JLValue.Str(s) => { return s; }
    _ => { return ""; }
  }
}
