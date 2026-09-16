// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-P03: Complex type graph -- 10 inter-dependent types + large match with 20 arms + deep nesting
// Pattern: fn main() -> Int { ... return 0; }

type Node = { id: Int; parent: Int; data: Data; }
type Data = { label: Str; value: Value; }
type Value = { kind: Kind; number: Int; }
type Kind = { code: Int; tag: Tag; }
type Tag = { name: Str; flags: Int; }
type Edge = { from: NodeRef; to: NodeRef; weight: Float64; }
type NodeRef = { index: Int; valid: Bool; }
type GraphInfo = { nodes: Int; edges: Int; root: Node; }
type Analysis = { sub: GraphInfo; score: Float64; kind: Kind; }
type Report = { summary: Str; analysis: Analysis; meta: Meta; }
type Meta = { author: Str; version: Int; created: Int; }

const TYPE_GRAPH_COUNT: Int = 10;

fn make_kind(c: Int, n: Str) -> Kind {
  return Kind{ code: c; tag: Tag{ name: n; flags: 0; }; };
}

fn make_value(k: Kind, num: Int) -> Value {
  return Value{ kind: k; number: num; };
}

fn make_data(l: Str, v: Value) -> Data {
  return Data{ label: l; value: v; };
}

fn make_node(id: Int, parent: Int, d: Data) -> Node {
  return Node{ id: id; parent: parent; data: d; };
}

fn build_graph() -> GraphInfo {
  var k = make_kind(1, "root");
  var v = make_value(k, 42);
  var d = make_data("root-node", v);
  var n = make_node(0, -1, d);
  return GraphInfo{ nodes: 1; edges: 0; root: n; };
}

fn large_match_20_arms(val: Int) -> Int {
  match val {
    0 => return 0,
    1 => return 1,
    2 => return 4,
    3 => return 9,
    4 => return 16,
    5 => return 25,
    6 => return 36,
    7 => return 49,
    8 => return 64,
    9 => return 81,
    10 => return 100,
    11 => return 121,
    12 => return 144,
    13 => return 169,
    14 => return 196,
    15 => return 225,
    16 => return 256,
    17 => return 289,
    18 => return 324,
    19 => return 361,
    _ => return -1,
  }
}

fn deep_10_level_nesting() -> Int {
  var result: Int = 0;
  var l1: Int = 1;
  var l2: Int = 2;
  var l3: Int = 3;
  var l4: Int = 4;
  var l5: Int = 5;
  var l6: Int = 6;
  var l7: Int = 7;
  var l8: Int = 8;
  var l9: Int = 9;
  var l10: Int = 10;
  result = l1 + l2 + l3 + l4 + l5 + l6 + l7 + l8 + l9 + l10;
  return result;
}

fn interleaved_chain_a() -> Int { return TYPE_GRAPH_COUNT; }

type ExtraA = { a: Int; b: Float64; }
fn interleaved_chain_b() -> Int { return interleaved_chain_a() + 1; }

enum ExtraB { Yes, No, Maybe(v: Int) }
fn interleaved_chain_c() -> Int { return interleaved_chain_b() + 1; }

type ExtraC = { x: Float64; y: Float64; z: Float64; w: Float64; }
fn interleaved_chain_d() -> Int { return interleaved_chain_c() + 1; }

enum ExtraD { Alpha(v: Str), Beta, Gamma(v: Int) }
fn interleaved_chain_e() -> Int { return interleaved_chain_d() + 1; }

const C_INT: Int = 100;
fn interleaved_chain_f() -> Int { return interleaved_chain_e() + 1; }

type ExtraE = { code: Int; msg: Str; ok: Bool; }
enum ExtraF { Fast, Slow(t: Int) }
fn interleaved_chain_g() -> Int { return interleaved_chain_f() + 1; }

const C_FLOAT: Float64 = 3.14;
type ExtraG = { min: Int; max: Int; step: Int; }
fn interleaved_chain_h() -> Int { return interleaved_chain_g() + 1; }

enum ExtraH { Low, Medium, High(v: Int) }
const C_CHAR: Char = 'Z';
fn interleaved_chain_i() -> Int { return interleaved_chain_h() + 1; }

type ExtraI = { head: Int; tail: Int; size: Int; }
enum ExtraJ { Ready, Running, Stopped, Error(v: Str) }
fn interleaved_chain_j() -> Int { return interleaved_chain_i() + 1; }

const C_STR: Str = "interleaved";
fn interleaved_chain_k() -> Int { return interleaved_chain_j() + 1; }

type ExtraK = { a: Int; b: Bool; c: Char; d: Float64; e: Str; }
fn interleaved_chain_l() -> Int { return interleaved_chain_k() + 1; }

const C_BOOL: Bool = true;
enum ExtraL { Empty, Full(v: Str, w: Int) }
fn interleaved_chain_m() -> Int { return interleaved_chain_l() + 1; }

type ExtraM = { id: Int; ref_id: Int; sum: Float64; }
const C_ZERO: Int = 0;
fn interleaved_chain_n() -> Int { return interleaved_chain_m() + 1; }

enum ExtraN { A, B(v: Int), C(v: Float64) }
type ExtraO = { val: Int; flag: Bool; }
fn interleaved_chain_o() -> Int { return interleaved_chain_n() + 1; }

module graph_utils {
  pub fn node_id(n: Node) -> Int { return n.id; }
  pub fn node_parent(n: Node) -> Int { return n.parent; }
}

module data_utils {
  use graph_utils.node_id;
  pub fn describe(n: Node) -> Int { return node_id(n) * 100; }
}

module calc_utils {
  pub fn sum_range(lo: Int, hi: Int) -> Int {
    var s: Int = 0;
    var i: Int = lo;
    while i <= hi { s = s + i; i = i + 1; }
    return s;
  }
}

module report_utils {
  use calc_utils.sum_range;
  use data_utils.describe;
  pub fn total_report(n: Node) -> Int {
    var base = describe(n);
    var extra = sum_range(1, 5);
    return base + extra;
  }
}

module verify_utils {
  pub fn check(v: Int, expected: Int) -> Int {
    if v == expected { return 0; }
    return 1;
  }
}

fn run_type_graph() -> Int {
  var g = build_graph();
  if g.nodes != 1 { return 1; }
  if g.edges != 0 { return 2; }
  if g.root.id != 0 { return 3; }
  if g.root.data.value.number != 42 { return 4; }
  return 0;
}

fn run_graph_utils() -> Int {
  var k = make_kind(1, "test");
  var v = make_value(k, 100);
  var d = make_data("test", v);
  var n = make_node(5, 0, d);
  if graph_utils.node_id(n) != 5 { return 1; }
  if graph_utils.node_parent(n) != 0 { return 2; }
  return 0;
}

fn run_cross_ref() -> Int {
  var k = make_kind(1, "test");
  var v = make_value(k, 100);
  var d = make_data("x", v);
  var n = make_node(2, -1, d);
  var r = report_utils.total_report(n);
  if r != 215 { return 1; }
  return 0;
}

fn main() -> Int {
  if run_type_graph() != 0 { return 1; }
  var lm = large_match_20_arms(10);
  if lm != 100 { return 2; }
  if large_match_20_arms(15) != 225 { return 3; }
  if large_match_20_arms(0) != 0 { return 4; }
  var nest = deep_10_level_nesting();
  if nest != 55 { return 5; }
  if interleaved_chain_o() != 24 { return 6; }
  if C_INT != 100 { return 7; }
  return 0;
}
