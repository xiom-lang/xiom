// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// m45_round14_aggregate_closure_params -- round-14 (2026-08-22) regression:
// closures whose parameters are AGGREGATE types (user structs, tuples,
// containers) corrupted on call -- the closure thunk declared every param
// as i64 while the call site passed the aggregate BY VALUE (a 16-byte
// struct splits across two registers; the i64 thunk param read only the
// first -- p.x garbage, tuple elements literal 0, &Vec params garbage).
// The thunk now declares aggregate params with their REAL LLVM types and
// binds them with those types; __fnwrap thunks forward aggregates by
// value. Scalar params (the uniform i64 convention) are unaffected.
module m45_round14_aggregate_closure_params
use xiom.core;

type Point = { x: Int; y: Int; }

fn apply_point(f: fn(Point) -> Bool) -> Bool {
  return f(Point{ x: 3; y: 4; });
}

fn apply_ref(f: fn(&Point) -> Bool) -> Bool {
  var p = Point{ x: 9; y: 1; };
  return f(&p);
}

fn apply_tuple(f: fn((Int, Int)) -> Bool) -> Bool {
  return f((11, 12));
}

fn apply_vec(f: fn(&Vec[Int]) -> Int) -> Int {
  var v = Vec[Int].new();
  v.push(7);
  v.push(8);
  return f(&v);
}

fn my_point_pred(p: Point) -> Bool {
  return p.x == 3;
}

fn main() -> Int {
  // struct param by value
  if !apply_point(fn(p: Point) -> Bool { return p.x == 3; }) { return 1; }
  // &struct param (struct-pointee ref)
  if !apply_ref(fn(p: &Point) -> Bool { return p.x == 9; }) { return 2; }
  // tuple param by value with field access
  if !apply_tuple(fn(p: (Int, Int)) -> Bool { return p.0 == 11 && p.1 == 12; }) { return 3; }
  // tuple param match-destructure
  if !apply_tuple(fn(p: (Int, Int)) -> Bool {
    match p {
      (a, b) => { return a == 11 && b == 12; },
    }
    return false;
  }) { return 4; }
  // &Vec[Int] param
  if apply_vec(fn(v: &Vec[Int]) -> Int { return v.len(); }) != 2 { return 5; }
  // fn-REFERENCE with an aggregate param (the __fnwrap path)
  if !apply_point(my_point_pred) { return 6; }
  return 0;
}
