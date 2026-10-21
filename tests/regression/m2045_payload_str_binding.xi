// m2045 part 2 (REL-20261019-2045-packages): Str payload bindings from a
// boxed Option/Result must carry the Str XIOM type.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `Vec.pop` boxes a non-scalar payload (malloc+memcpy+ptrtoint) and the
// match-arm rebind must dereference/type it. Before this fix:
//   - `Vec[Result[Str, Str]]` pop -> `Some(r)` -> `Err(msg)` bound msg as i64
//     (printed the Str pointer as a number);
//   - `Vec[S].pop()` with `type S = Str` lost the payload type entirely
//     (scalar elements were never tracked for pop), same pointer print.
// The fix tracks the alias-expanded element type for pop/get/remove, resolves
// the Result error payload from a local's declared type, and records the XIOM
// payload type on a boxed-struct match binding.

module m2045_payload_str_binding

use xiom.io;
use xiom.convert;

type S = Str;
type ER = Result[Str, Str];

fn direct_err() -> Int {
  var v = Vec[ER].new();
  v.push(Err("direct-message"));
  if v.len() != 1 { return 1; }
  let e = v.pop();
  match e {
    Some(r) => {
      match r {
        Err(msg) => {
          io.println("direct=" + msg);
          if msg != "direct-message" { return 2; }
        }
        Ok(_) => { return 3; }
      }
    }
    None => { return 4; }
  }
  return 0;
}

fn local_err() -> Int {
  var v = Vec[ER].new();
  let local: ER = Err("local-message");
  v.push(local);
  let e = v.pop();
  match e {
    Some(r) => {
      match r {
        Err(msg) => {
          io.println("local=" + msg);
          if msg != "local-message" { return 5; }
        }
        Ok(_) => { return 6; }
      }
    }
    None => { return 7; }
  }
  return 0;
}

fn alias_str_pop() -> Int {
  var v = Vec[S].new();
  v.push("alias-str");
  let e = v.pop();
  match e {
    Some(s) => {
      io.println("alias=" + s);
      if s != "alias-str" { return 8; }
    }
    None => { return 9; }
  }
  return 0;
}

fn annotated_result() -> Int {
  var v = Vec[Result[Str, Str]].new();
  v.push(Err("annot-message"));
  let o: Option[Result[Str, Str]] = v.pop();
  match o {
    Some(r) => {
      match r {
        Err(msg) => {
          io.println("annot=" + msg);
          if msg != "annot-message" { return 10; }
        }
        Ok(_) => { return 11; }
      }
    }
    None => { return 12; }
  }
  return 0;
}

fn main() -> Int {
  let a = direct_err();
  if a != 0 { return a; }
  let b = local_err();
  if b != 0 { return b; }
  let c = alias_str_pop();
  if c != 0 { return c; }
  let d = annotated_result();
  if d != 0 { return d; }
  io.println("m2045 ok i=" + convert.int_to_string(a + b + c + d));
  return 0;
}
