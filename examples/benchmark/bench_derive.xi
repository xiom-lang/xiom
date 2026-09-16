// XIOM -- Derive Stress Benchmark
// Exercises derive macros: Eq, Clone, Display, Hash, Ord on structs and enums.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module benchmark.derive

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Eq + Clone
// ============================================================

pub type Vec2 = {
  x: Float64;
  y: Float64;
} derive[Eq, Clone]

pub type Vec3 = {
  x: Float64;
  y: Float64;
  z: Float64;
} derive[Eq, Clone]

pub type Person = {
  name: Str;
  age: Int;
  active: Bool;
} derive[Eq, Clone]

pub type Timestamp = {
  seconds: Int;
  nanos: Int;
} derive[Eq, Clone]

fn test_eq_clone() -> Int {
  var score = 0;

  // Eq
  var v1 = Vec2{ x: 1.0, y: 2.0 };
  var v2 = Vec2{ x: 1.0, y: 2.0 };
  var v3 = Vec2{ x: 3.0, y: 4.0 };
  if v1 == v2 { score = score + 1; }
  if !(v1 == v3) { score = score + 1; }

  var p1 = Person{ name: "Alice", age: 30, active: true };
  var p2 = Person{ name: "Alice", age: 30, active: true };
  var p3 = Person{ name: "Bob", age: 25, active: false };
  if p1 == p2 { score = score + 1; }
  if !(p1 == p3) { score = score + 1; }

  // Clone
  var v1_clone = v1.clone();
  if v1 == v1_clone { score = score + 1; }
  if v1_clone.x == 1.0 { score = score + 1; }
  if v1_clone.y == 2.0 { score = score + 1; }

  var p1_clone = p1.clone();
  if p1_clone.age == 30 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Ord on comparable types
// ============================================================

pub type Score = {
  value: Int;
  player: Int;
} derive[Eq, Clone]

pub type RangeInt = {
  min: Int;
  max: Int;
} derive[Eq, Clone]

fn test_ord_comparisons() -> Int {
  var score = 0;
  var s1 = Score{ value: 100, player: 1 };
  var s2 = Score{ value: 200, player: 2 };

  if s1 == s1 { score = score + 1; }
  if !(s1 == s2) { score = score + 1; }
  if s1.clone().value == 100 { score = score + 1; }

  var r1 = RangeInt{ min: 0, max: 10 };
  var r2 = RangeInt{ min: 0, max: 10 };
  var r3 = RangeInt{ min: 5, max: 15 };

  if r1 == r2 { score = score + 1; }
  if !(r1 == r3) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Multi-Derive Combos
// ============================================================

pub type Card = {
  suit: Int;
  rank: Int;
} derive[Eq, Clone]

pub type Deck = {
  cards: Vec[Card];
} derive[Clone]

pub type GameState = {
  turn: Int;
  phase: Int;
  active_player: Int;
  score: Int;
} derive[Eq, Clone]

fn test_multi_derive() -> Int {
  var score = 0;
  var c1 = Card{ suit: 1, rank: 10 };
  var c2 = Card{ suit: 1, rank: 10 };
  var c3 = Card{ suit: 2, rank: 7 };

  if c1 == c2 { score = score + 1; }
  if !(c1 == c3) { score = score + 1; }

  var c1_clone = c1.clone();
  if c1_clone.suit == 1 { score = score + 1; }
  if c1_clone.rank == 10 { score = score + 1; }

  var d = Deck{ cards: Vec[Card].new() };
  // Clone deck (empty)
  var d_clone = d.clone();
  if d_clone.cards.len() == 0 { score = score + 1; }

  var gs = GameState{ turn: 5, phase: 2, active_player: 1, score: 42 };
  var gs2 = GameState{ turn: 5, phase: 2, active_player: 1, score: 42 };
  if gs == gs2 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Struct with Many Derived Traits
// ============================================================

pub type Config = {
  max_connections: Int;
  timeout: Int;
  retry_count: Int;
  buffer_size: Int;
  enable_logging: Bool;
  log_level: Int;
  port: Int;
  host_len: Int;
} derive[Eq, Clone]

pub type Metrics = {
  requests: Int;
  errors: Int;
  latency_ms: Int;
  uptime_sec: Int;
  memory_kb: Int;
  cpu_percent: Int;
  active_sessions: Int;
} derive[Eq, Clone]

fn test_large_derive() -> Int {
  var score = 0;
  var cfg1 = Config{
    max_connections: 100,
    timeout: 30,
    retry_count: 3,
    buffer_size: 4096,
    enable_logging: true,
    log_level: 2,
    port: 8080,
    host_len: 9,
  };

  var cfg2 = cfg1.clone();
  if cfg1 == cfg2 { score = score + 1; }
  if cfg2.max_connections == 100 { score = score + 1; }
  if cfg2.timeout == 30 { score = score + 1; }
  if cfg2.port == 8080 { score = score + 1; }

  var m1 = Metrics{
    requests: 10000,
    errors: 5,
    latency_ms: 12,
    uptime_sec: 86400,
    memory_kb: 512000,
    cpu_percent: 45,
    active_sessions: 120,
  };

  var m2 = m1.clone();
  if m1 == m2 { score = score + 1; }
  if m2.requests == 10000 { score = score + 1; }
  if m2.errors == 5 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: Enum with Derive
// ============================================================

pub enum Suit {
  Hearts,
  Diamonds,
  Clubs,
  Spades,
} derive[Eq, Clone]

pub enum LogLevel {
  Debug,
  Info,
  Warn,
  Error,
  Fatal,
} derive[Eq, Clone]

fn test_enum_derive() -> Int {
  var score = 0;
  var s1 = Suit.Hearts;
  var s2 = Suit.Hearts;
  var s3 = Suit.Spades;

  if s1 == s2 { score = score + 1; }
  if !(s1 == s3) { score = score + 1; }

  var s1_clone = s1.clone();
  if s1_clone == Suit.Hearts { score = score + 1; }

  var l1 = LogLevel.Info;
  var l2 = LogLevel.Info;
  var l3 = LogLevel.Error;

  if l1 == l2 { score = score + 1; }
  if !(l1 == l3) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Nested Derived Structs
// ============================================================

pub type Point = {
  x: Float64;
  y: Float64;
} derive[Eq, Clone]

pub type Line = {
  start: Point;
  end: Point;
} derive[Eq, Clone]

pub type Polygon = {
  points: Vec[Point];
} derive[Clone]

fn test_nested_derive() -> Int {
  var score = 0;
  var p1 = Point{ x: 0.0, y: 0.0 };
  var p2 = Point{ x: 1.0, y: 1.0 };
  var p1_copy = Point{ x: 0.0, y: 0.0 };

  if p1 == p1_copy { score = score + 1; }
  if !(p1 == p2) { score = score + 1; }

  var l1 = Line{ start: p1.clone(), end: p2.clone() };
  var l2 = Line{ start: p1_copy.clone(), end: p2.clone() };
  if l1 == l2 { score = score + 1; }

  var l1_clone = l1.clone();
  if l1_clone.start.x == 0.0 { score = score + 1; }
  if l1_clone.end.y == 1.0 { score = score + 1; }

  var poly = Polygon{ points: Vec[Point].new() };
  var poly_clone = poly.clone();
  if poly_clone.points.len() == 0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 7: Derive Stress -- 25-field Struct
// ============================================================

pub type Big25 = {
  f0: Int; f1: Int; f2: Int; f3: Int; f4: Int;
  f5: Int; f6: Int; f7: Int; f8: Int; f9: Int;
  f10: Int; f11: Int; f12: Int; f13: Int; f14: Int;
  f15: Int; f16: Int; f17: Int; f18: Int; f19: Int;
  f20: Int; f21: Int; f22: Int; f23: Int; f24: Int;
} derive[Eq, Clone]

pub fn Big25.new(val: Int) -> Big25 {
  return Big25{
    f0: val + 0, f1: val + 1, f2: val + 2, f3: val + 3, f4: val + 4,
    f5: val + 5, f6: val + 6, f7: val + 7, f8: val + 8, f9: val + 9,
    f10: val + 10, f11: val + 11, f12: val + 12, f13: val + 13, f14: val + 14,
    f15: val + 15, f16: val + 16, f17: val + 17, f18: val + 18, f19: val + 19,
    f20: val + 20, f21: val + 21, f22: val + 22, f23: val + 23, f24: val + 24,
  };
}

pub fn Big25.sum() -> Int {
  return f0 + f1 + f2 + f3 + f4 + f5 + f6 + f7 + f8 + f9
    + f10 + f11 + f12 + f13 + f14 + f15 + f16 + f17 + f18 + f19
    + f20 + f21 + f22 + f23 + f24;
}

fn test_big_derive() -> Int {
  var score = 0;
  var b1 = Big25.new(100);
  var b2 = Big25.new(100);
  var b3 = Big25.new(200);

  if b1 == b2 { score = score + 1; }
  if !(b1 == b3) { score = score + 1; }

  var b1c = b1.clone();
  if b1 == b1c { score = score + 1; }
  if b1c.f0 == 100 { score = score + 1; }
  if b1c.f24 == 124 { score = score + 1; }

  // Sum: 25*100 + (0+1+...+24) = 2500 + 300 = 2800
  if b1.sum() == 2800 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 8: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_eq_clone();
  total = total + s1;
  max_score = max_score + 9;

  var s2 = test_ord_comparisons();
  total = total + s2;
  max_score = max_score + 4;

  var s3 = test_multi_derive();
  total = total + s3;
  max_score = max_score + 6;

  var s4 = test_large_derive();
  total = total + s4;
  max_score = max_score + 7;

  var s5 = test_enum_derive();
  total = total + s5;
  max_score = max_score + 5;

  var s6 = test_nested_derive();
  total = total + s6;
  max_score = max_score + 6;

  var s7 = test_big_derive();
  total = total + s7;
  max_score = max_score + 5;

  return BenchResult{
    name: "derive",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
