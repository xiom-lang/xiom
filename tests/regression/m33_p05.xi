// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-P05: Multi-module cross-reference stress + interleaved declarations + deep call chain
// Pattern: fn main() -> Int { ... return 0; }

const BASE0: Int = 0;
const BASE1: Int = 1;

module registry {
  pub type Entry = { id: Int; name: Str; active: Bool; }
  pub fn make_entry(id: Int, name: Str) -> Entry {
    return Entry{ id: id; name: name; active: true; };
  }
  pub fn entry_id(e: Entry) -> Int { return e.id; }
}

module store {
  use registry.Entry;
  use registry.entry_id;
  pub type Store = { items: Int; capacity: Int; }
  pub fn add_item(s: Store, e: Entry) -> Store {
    var new_items: Int = s.items + 1;
    return Store{ items: new_items; capacity: s.capacity; };
  }
  pub fn has_capacity(s: Store) -> Bool { return s.items < s.capacity; }
}

module catalog {
  use registry.Entry;
  use registry.make_entry;
  use store.Store;
  use store.add_item;
  use store.has_capacity;
  pub fn process(s: Store) -> Store {
    var e1 = make_entry(1, "alpha");
    var e2 = make_entry(2, "beta");
    var s1 = add_item(s, e1);
    var s2 = add_item(s1, e2);
    return s2;
  }
}

module validators {
  use registry.Entry;
  pub fn validate_entry(e: Entry) -> Bool {
    if e.id < 0 { return false; }
    if e.active == false { return false; }
    return true;
  }
}

module aggregator {
  use registry.entry_id;
  use store.Store;
  use store.add_item;
  use catalog.process;
  use validators.validate_entry;
  pub fn run_workflow() -> Int {
    var s = Store{ items: 0; capacity: 10; };
    var result = process(s);
    return result.items;
  }
}

fn call_chain_s0() -> Int { return BASE0; }
fn call_chain_s1() -> Int { return call_chain_s0() + 1; }
fn call_chain_s2() -> Int { return call_chain_s1() + 2; }

type Tp5_A = { x: Int; y: Int; }
fn call_chain_s3() -> Int { return call_chain_s2() + 3; }

enum Ep5_A { One, Two, Three(v: Int) }
fn call_chain_s4() -> Int { return call_chain_s3() + 4; }

type Tp5_B = { val: Float64; tag: Str; }
fn call_chain_s5() -> Int { return call_chain_s4() + 5; }

enum Ep5_B { Good, Bad(code: Int) }
fn call_chain_s6() -> Int { return call_chain_s5() + 6; }

const CP5_A: Int = 10;
type Tp5_C = { a: Bool; b: Char; c: Int; }
fn call_chain_s7() -> Int { return call_chain_s6() + 7; }

enum Ep5_C { Alpha, Beta, Gamma, Delta, Epsilon }
fn call_chain_s8() -> Int { return call_chain_s7() + 8; }

type Tp5_D = { min: Float64; max: Float64; avg: Float64; }
fn call_chain_s9() -> Int { return call_chain_s8() + 9; }

enum Ep5_D { Small(v: Int), Large(v: Int, w: Int) }
fn call_chain_s10() -> Int { return call_chain_s9() + 10; }

const CP5_B: Int = 20;
type Tp5_E = { head: Int; tail: Int; size: Int; }
fn call_chain_s11() -> Int { return call_chain_s10() + 11; }

enum Ep5_E { North, South, East, West }
fn call_chain_s12() -> Int { return call_chain_s11() + 12; }

type Tp5_F = { id: Int; parent: Int; level: Int; }
fn call_chain_s13() -> Int { return call_chain_s12() + 13; }

enum Ep5_F { Pass, Fail(reason: Str) }
fn call_chain_s14() -> Int { return call_chain_s13() + 14; }

const CP5_C: Int = 30;
type Tp5_G = { key: Str; value: Int; flag: Bool; }
fn call_chain_s15() -> Int { return call_chain_s14() + 15; }

enum Ep5_G { Red, Green, Blue, Yellow, Cyan, Magenta }
fn call_chain_s16() -> Int { return call_chain_s15() + 16; }

type Tp5_H = { row: Int; col: Int; data: Float64; }
fn call_chain_s17() -> Int { return call_chain_s16() + 17; }

enum Ep5_H { Success(v: Int), Failure(v: Int, msg: Str) }
fn call_chain_s18() -> Int { return call_chain_s17() + 18; }

type Tp5_I = { version: Int; build: Int; timestamp: Int; }
fn call_chain_s19() -> Int { return call_chain_s18() + 19; }

enum Ep5_I { Valid, Invalid(reason: Int) }
fn call_chain_s20() -> Int { return call_chain_s19() + 20; }

const CP5_D: Int = 40;
type Tp5_J = { total: Int; items: Int; rate: Float64; }
fn call_chain_s21() -> Int { return call_chain_s20() + 21; }

enum Ep5_J { Hot, Cold, Warm(temp: Float64) }
fn call_chain_s22() -> Int { return call_chain_s21() + 22; }

type Tp5_K = { x1: Float64; y1: Float64; x2: Float64; y2: Float64; }
fn call_chain_s23() -> Int { return call_chain_s22() + 23; }

enum Ep5_K { Active, Inactive, Pending, Cancelled }
fn call_chain_s24() -> Int { return call_chain_s24_inner() + 0; }

fn call_chain_s24_inner() -> Int { return call_chain_s23() + 24; }

const CP5_E: Int = 50;
type Tp5_L = { width: Int; height: Int; depth: Float64; }
fn call_chain_s25() -> Int { return call_chain_s24() + 25; }

enum Ep5_L { Ready, Running, Stopped }
fn call_chain_s26() -> Int { return call_chain_s25() + 26; }

type Tp5_M = { a1: Int; a2: Int; a3: Float64; a4: Bool; a5: Char; }
fn call_chain_s27() -> Int { return call_chain_s26() + 27; }

enum Ep5_M { A(v: Int), B, C(v: Float64) }
fn call_chain_s28() -> Int { return call_chain_s27() + 28; }

type Tp5_N = { start: Int; end: Int; step: Int; }
fn call_chain_s29() -> Int { return call_chain_s28() + 29; }

const CP5_F: Int = 60;
const CP5_G: Int = 70;
const CP5_H: Int = 80;
const CP5_I: Int = 90;
const CP5_J: Int = 100;

enum Ep5_N { Single(v: Int), Double(v: Int, w: Int), Triple(v: Int, w: Int, z: Int) }
type Tp5_O = { r: Int; g: Int; b: Int; a: Int; }
type Tp5_P = { source: Str; dest: Str; port: Int; }
enum Ep5_O { HTTP, HTTPS, TCP, UDP, Custom(name: Str) }
type Tp5_Q = { hash: Int; salt: Int; rounds: Int; }
type Tp5_R = { lt: Float64; ln: Float64; alt: Float64; }
enum Ep5_P { Verified, Unverified, Expired, Revoked }
type Tp5_S = { count: Int; sum: Int; mean: Float64; variance: Float64; }
type Tp5_T = { id: Int; revision: Int; patch: Int; }

fn struct_chain_test() -> Int {
  var a = Tp5_A{ x: 1; y: 2; };
  var b = Tp5_C{ a: true; b: 'X'; c: 3; };
  var c = Tp5_J{ total: 100; items: 10; rate: 2.5; };
  var d = Tp5_M{ a1: 1; a2: 2; a3: 3.0; a4: true; a5: 'Z'; };
  var s = a.x + a.y + b.c + c.total + d.a1 + d.a2;
  if s != 109 { return 1; }
  return 0;
}

fn enum_payload_test() -> Int {
  var e = Ep5_H.Success(42);
  match e {
    Ep5_H.Success(v) => if v == 42 { return 0; } else { return 1; },
    Ep5_H.Failure(c, m) => return c,
  }
}

fn main() -> Int {
  var chain_val = call_chain_s29();
  if chain_val != 435 { return 1; }
  if struct_chain_test() != 0 { return 2; }
  if enum_payload_test() != 0 { return 3; }
  if CP5_B != 20 { return 4; }
  if CP5_J != 100 { return 5; }
  return 0;
}
