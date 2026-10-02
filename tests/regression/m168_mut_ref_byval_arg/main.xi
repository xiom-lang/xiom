// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m168 residual lock (stdlib/packages narrowing): a `&mut T` PARAM passed to
// a BY-VALUE T param in a DIRECT call must load the pointee value. Before the
// fix, coerce_ref_arg_to_pointee was only consulted inside the pointer-param
// branch and only matched the "*T" form -- `local_xiom_types` records the
// ref-preserving "&mut T" -- so `byval(s)` ptrtoint'd the ADDRESS and bump's
// write stored address-bits+1 (shuffle seed never advanced).
module m168_mut_ref_byval_arg

fn byval(x: Int) -> Int { return x + 1; }

fn bump(s: &mut Int) {
  let v = byval(s);
  s = v;
}

fn veclen(v: Vec[Int]) -> Int { return v.len(); }

// Container-pointee sibling: the arg must load the %struct.Vec header, not
// inttoptr the header address.
fn rehead(s: &mut Vec[Int]) {
  let n = veclen(s);
  var w = Vec[Int].new();
  var i = 0;
  while i < n {
    w.push(100 + i);
    i = i + 1;
  }
  s = w;
}

fn main() -> Int {
  var st: Int = 10;
  bump(&mut st);
  if st != 11 { return 1; }

  var v = Vec[Int].new();
  v.push(1);
  v.push(2);
  rehead(&mut v);
  if v.len() != 2 { return 2; }
  if v[0] != 100 { return 3; }
  if v[1] != 101 { return 4; }
  return 0;
}
