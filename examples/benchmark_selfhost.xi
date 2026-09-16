// XIOM -- Selfhost Compiler Benchmark
// Exercises all language features at scale.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

// === MODULE 1: Math Library ===
module math_lib {
  pub fn factorial(n: Int) -> Int {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
  }

  pub fn fibonacci(n: Int) -> Int {
    if n <= 1 { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
  }

  pub fn is_prime(n: Int) -> Bool {
    if n < 2 { return false; }
    var i = 2;
    while i * i < n {
      if n % i == 0 { return false; }
      i = i + 1;
    }
    return true;
  }

  pub fn gcd(a: Int, b: Int) -> Int {
    if b == 0 { return a; }
    return gcd(b, a % b);
  }

  pub fn power(base: Int, exp: Int) -> Int {
    if exp == 0 { return 1; }
    return base * power(base, exp - 1);
  }

  pub fn abs(n: Int) -> Int {
    if n >= 0 { return n; }
    return -n;
  }

  pub fn max(a: Int, b: Int) -> Int {
    if a > b { return a; }
    return b;
  }

  pub fn min(a: Int, b: Int) -> Int {
    if a < b { return a; }
    return b;
  }

  pub fn collatz(n: Int) -> Int {
    if n <= 1 { return 0; }
    if n % 2 == 0 { return 1 + collatz(n / 2); }
    return 1 + collatz(3 * n + 1);
  }

  pub fn triangular(n: Int) -> Int {
    if n <= 0 { return 0; }
    return n + triangular(n - 1);
  }

  pub fn digit_sum(n: Int) -> Int {
    if n < 10 { return n; }
    return n % 10 + digit_sum(n / 10);
  }

  pub fn count_divisors(n: Int) -> Int {
    if n <= 0 { return 0; }
    var count = 0;
    var i = 1;
    while i < n + 1 {
      if n % i == 0 { count = count + 1; }
      i = i + 1;
    }
    return count;
  }

  pub fn is_even(n: Int) -> Bool {
    return n % 2 == 0;
  }

  pub fn is_odd(n: Int) -> Bool {
    return n % 2 != 0;
  }

  pub fn clamp(val: Int, lo: Int, hi: Int) -> Int {
    if val < lo { return lo; }
    if val > hi { return hi; }
    return val;
  }

  pub fn lerp(a: Int, b: Int, t: Int) -> Int {
    return a + (b - a) * t;
  }
}

// === MODULE 2: Predicate Combinators ===
module predicates {
  pub fn is_positive(n: Int) -> Bool {
    return n > 0;
  }

  pub fn is_negative(n: Int) -> Bool {
    return n < 0;
  }

  pub fn is_zero(n: Int) -> Bool {
    return n == 0;
  }

  pub fn is_non_zero(n: Int) -> Bool {
    return n != 0;
  }

  pub fn both(a: Bool, b: Bool) -> Bool {
    return a && b;
  }

  pub fn either(a: Bool, b: Bool) -> Bool {
    return a || b;
  }

  pub fn negate(b: Bool) -> Bool {
    return !b;
  }

  pub fn in_range(val: Int, lo: Int, hi: Int) -> Bool {
    return val >= lo && val < hi;
  }

  pub fn is_multiple_of(a: Int, b: Int) -> Bool {
    if b == 0 { return false; }
    return a % b == 0;
  }

  pub fn implies(a: Bool, b: Bool) -> Bool {
    return !a || b;
  }
}

// === MODULE 3: Control Flow Patterns ===
module control {
  pub fn count_rec(limit: Int, acc: Int) -> Int {
    if limit <= 0 { return acc; }
    return count_rec(limit - 1, acc + limit - 1);
  }

  pub fn classify(n: Int) -> Int {
    if n > 0 { return 1; }
    elif n < 0 { return -1; }
    else { return 0; }
  }

  pub fn fizzbuzz(n: Int) -> Int {
    if n % 15 == 0 { return 0; }
    if n % 3 == 0 { return 1; }
    if n % 5 == 0 { return 2; }
    return 3;
  }

  pub fn match_kind(n: Int) -> Int {
    match n {
      0 => 10,
      1 => 20,
      2 => 30,
      _ => 0,
    }
  }

  pub fn match_bool(b: Bool) -> Int {
    match b {
      true => 1,
      false => 0,
    }
  }

  pub fn while_sum(limit: Int) -> Int {
    var total = 0;
    var i = 0;
    while i < limit + 1 {
      if true { total = total + i; }
      i = i + 1;
    }
    return total;
  }

  pub fn count_to(limit: Int) -> Int {
    var i = 0;
    while i < limit {
      if true { i = i + 1; }
    }
    return i;
  }
}

// === MODULE 4: Data Structures ===
module structures {
  pub type Point = {
    x: Float64;
    y: Float64;
  } derive[Eq, Clone]

  pub type Line = {
    start: Point;
    end: Point;
  } derive[Eq, Clone]

  pub type Circle = {
    center: Point;
    radius: Float64;
  } derive[Eq, Clone]

  pub type Rectangle = {
    x: Float64;
    y: Float64;
    w: Float64;
    h: Float64;
  } derive[Eq, Clone]

  pub fn make_point(x: Float64, y: Float64) -> Point {
    return Point{ x: x, y: y };
  }

  pub fn make_line(x1: Float64, y1: Float64, x2: Float64, y2: Float64) -> Line {
    var p1 = make_point(x1, y1);
    var p2 = make_point(x2, y2);
    return Line{ start: p1, end: p2 };
  }

  pub fn distance(a: &Point, b: &Point) -> Float64 {
    var dx = a.x - b.x;
    var dy = a.y - b.y;
    return dx * dx + dy * dy;
  }

  pub fn circle_area(c: &Circle) -> Float64 {
    return 3.14159 * c.radius * c.radius;
  }

  pub fn rect_area(r: &Rectangle) -> Float64 {
    return r.w * r.h;
  }

  pub fn rect_contains(r: &Rectangle, px: Float64, py: Float64) -> Bool {
    if px >= r.x && px < r.x + r.w {
      if py >= r.y && py < r.y + r.h { return true; }
    }
    return false;
  }

  pub fn midpoint(a: &Point, b: &Point) -> Point {
    var mx = (a.x + b.x) / 2.0;
    var my = (a.y + b.y) / 2.0;
    return Point{ x: mx, y: my };
  }

  pub fn scale_point(p: &Point, factor: Float64) -> Point {
    return Point{ x: p.x * factor, y: p.y * factor };
  }

  pub fn add_points(a: &Point, b: &Point) -> Point {
    return Point{ x: a.x + b.x, y: a.y + b.y };
  }

  pub fn dot_product(a: &Point, b: &Point) -> Float64 {
    return a.x * b.x + a.y * b.y;
  }

  pub fn point_eq(a: &Point, b: &Point) -> Bool {
    return a.x == b.x && a.y == b.y;
  }
}

// === MODULE 5: Generic Utilities ===
module generics_mod {
  pub fn identity[T](x: T) -> T {
    return x;
  }

  pub fn wrap_some[T](x: T) -> Option[T] {
    return Some(x);
  }

  pub fn unwrap_or[T](opt: Option[T], default: T) -> T {
    if opt.is_some { return opt.value; }
    return default;
  }

  pub fn result_ok[T, E](x: T) -> Result[T, E] {
    return Ok(x);
  }

  pub fn result_err[T, E](e: E) -> Result[T, E] {
    return Err(e);
  }
}

// === MODULE 6: Error Handling ===
module errors_mod {
  pub type MathError = {
    code: Int;
    msg: Str;
  } derive[Eq, Clone]

  pub fn safe_mod(a: Int, b: Int) -> Result[Int, MathError] {
    if b == 0 {
      return Err(MathError{ code: 2, msg: "mod by zero" });
    }
    return Ok(a % b);
  }

  pub fn safe_div(a: Int, b: Int) -> Result[Int, MathError] {
    if b == 0 {
      return Err(MathError{ code: 1, msg: "div by zero" });
    }
    return Ok(a / b);
  }

  pub fn safe_sub(a: Int, b: Int) -> Result[Int, MathError] {
    if a < b {
      return Err(MathError{ code: 3, msg: "underflow" });
    }
    return Ok(a - b);
  }
}

// === MODULE 7: Contracts ===
module contracts_mod {
  pub type Counter = {
    value: Int;
    lo: Int;
    hi: Int;
    invariant: value >= lo;
    invariant: value <= hi;
  }

  pub fn make_counter(val: Int, lo: Int, hi: Int) -> Counter
    requires: val >= lo
    requires: val <= hi
  {
    return Counter{ value: val, lo: lo, hi: hi };
  }

  pub fn counter_inc(c: &Counter) -> Counter
    requires: c.value < c.hi
    ensures: result.value == c.value + 1
  {
    return Counter{ value: c.value + 1, lo: c.lo, hi: c.hi };
  }

  pub fn counter_dec(c: &Counter) -> Counter
    requires: c.value > c.lo
    ensures: result.value == c.value - 1
  {
    return Counter{ value: c.value - 1, lo: c.lo, hi: c.hi };
  }

  pub fn safe_sqrt(x: Float64) -> Float64
    requires: x >= 0.0
  {
    return x;
  }
}

// === USE IMPORTS ===
use math_lib.factorial;
use math_lib.fibonacci;
use math_lib.is_prime;
use math_lib.gcd;
use math_lib.power;
use math_lib.abs;
use math_lib.max;
use math_lib.min;
use math_lib.collatz;
use math_lib.triangular;
use math_lib.digit_sum;
use math_lib.count_divisors;
use math_lib.is_even;
use math_lib.is_odd;
use math_lib.clamp;
use math_lib.lerp;
use control.count_rec;
use control.classify;
use control.fizzbuzz;
use control.match_kind;
use control.match_bool;
use control.while_sum;
use control.count_to;
use structures.make_point;
use structures.make_line;
use structures.distance;
use structures.circle_area;
use structures.rect_area;
use structures.rect_contains;
use structures.midpoint;
use structures.scale_point;
use structures.add_points;
use structures.dot_product;
use structures.point_eq;
use structures.Point;
use structures.Line;
use structures.Circle;
use structures.Rectangle;
use generics_mod.identity;
use generics_mod.wrap_some;
use generics_mod.unwrap_or;
use generics_mod.result_ok;
use generics_mod.result_err;
use errors_mod.MathError;
use errors_mod.safe_mod;
use errors_mod.safe_div;
use errors_mod.safe_sub;
use contracts_mod.Counter;
use contracts_mod.make_counter;
use contracts_mod.counter_inc;
use contracts_mod.counter_dec;
use contracts_mod.safe_sqrt;
use predicates.is_positive;
use predicates.is_negative;
use predicates.is_zero;
use predicates.is_non_zero;
use predicates.both;
use predicates.either;
use predicates.negate;
use predicates.in_range;
use predicates.is_multiple_of;
use predicates.implies;

// === MAIN BENCHMARK FUNCTIONS ===

fn run_math() -> Int {
  var score = 0;
  if factorial(5) == 120 { score = score + 1; }
  if fibonacci(10) == 55 { score = score + 1; }
  if is_prime(17) { score = score + 1; }
  if !(is_prime(4)) { score = score + 1; }
  if gcd(48, 18) == 6 { score = score + 1; }
  if power(2, 10) == 1024 { score = score + 1; }
  if abs(-5) == 5 { score = score + 1; }
  if max(10, 20) == 20 { score = score + 1; }
  if min(10, 20) == 10 { score = score + 1; }
  if collatz(6) == 8 { score = score + 1; }
  if triangular(10) == 55 { score = score + 1; }
  if digit_sum(123) == 6 { score = score + 1; }
  if count_divisors(12) == 6 { score = score + 1; }
  if is_even(42) { score = score + 1; }
  if is_odd(13) { score = score + 1; }
  if clamp(5, 0, 10) == 5 { score = score + 1; }
  if clamp(-1, 0, 10) == 0 { score = score + 1; }
  if clamp(15, 0, 10) == 10 { score = score + 1; }
  if lerp(0, 10, 3) == 30 { score = score + 1; }
  return score;
}

fn run_predicates() -> Int {
  var score = 0;
  if is_positive(5) { score = score + 1; }
  if is_negative(-3) { score = score + 1; }
  if is_zero(0) { score = score + 1; }
  if is_non_zero(42) { score = score + 1; }
  if both(true, true) { score = score + 1; }
  if !(both(true, false)) { score = score + 1; }
  if either(true, false) { score = score + 1; }
  if !(either(false, false)) { score = score + 1; }
  if negate(false) { score = score + 1; }
  if in_range(5, 0, 10) { score = score + 1; }
  if !(in_range(15, 0, 10)) { score = score + 1; }
  if is_multiple_of(10, 5) { score = score + 1; }
  if !(is_multiple_of(10, 3)) { score = score + 1; }
  if implies(true, true) { score = score + 1; }
  if implies(false, false) { score = score + 1; }
  if !(implies(true, false)) { score = score + 1; }
  return score;
}

fn run_control() -> Int {
  var score = 0;
  if count_rec(10, 0) == 45 { score = score + 1; }
  if classify(5) == 1 { score = score + 1; }
  if classify(-3) == -1 { score = score + 1; }
  if classify(0) == 0 { score = score + 1; }
  if fizzbuzz(15) == 0 { score = score + 1; }
  if fizzbuzz(3) == 1 { score = score + 1; }
  if fizzbuzz(5) == 2 { score = score + 1; }
  if fizzbuzz(7) == 3 { score = score + 1; }
  if match_kind(0) == 10 { score = score + 1; }
  if match_kind(1) == 20 { score = score + 1; }
  if match_kind(99) == 0 { score = score + 1; }
  if match_bool(true) == 1 { score = score + 1; }
  if match_bool(false) == 0 { score = score + 1; }
  if while_sum(10) == 55 { score = score + 1; }
  if count_to(10) == 10 { score = score + 1; }
  return score;
}

fn run_structures() -> Int {
  var p1 = make_point(0.0, 0.0);
  var p2 = make_point(3.0, 4.0);
  var score = 0;
  if p1.x == 0.0 { score = score + 1; }
  if p2.y == 4.0 { score = score + 1; }

  var d = distance(&p1, &p2);
  if d == 25.0 { score = score + 1; }

  var line = make_line(0.0, 0.0, 1.0, 1.0);
  if line.start.x == 0.0 { score = score + 1; }
  if line.end.y == 1.0 { score = score + 1; }

  var circ = Circle{ center: p1, radius: 5.0 };
  if circle_area(&circ) > 78.5 { score = score + 1; }

  var rect = Rectangle{ x: 0.0, y: 0.0, w: 10.0, h: 20.0 };
  if rect_area(&rect) == 200.0 { score = score + 1; }
  if rect_contains(&rect, 5.0, 10.0) { score = score + 1; }
  if !(rect_contains(&rect, 15.0, 10.0)) { score = score + 1; }

  var mp = midpoint(&p1, &p2);
  if mp.x == 1.5 { score = score + 1; }
  if mp.y == 2.0 { score = score + 1; }

  var p3 = scale_point(&p2, 2.0);
  if p3.x == 6.0 { score = score + 1; }
  if p3.y == 8.0 { score = score + 1; }

  var p4 = add_points(&p1, &p2);
  if p4.x == 3.0 { score = score + 1; }

  var dp = dot_product(&p2, &p2);
  if dp == 25.0 { score = score + 1; }

  if point_eq(&p1, &p1) { score = score + 1; }
  if !(point_eq(&p1, &p2)) { score = score + 1; }

  return score;
}

fn run_generics() -> Int {
  var score = 0;
  if identity(42) == 42 { score = score + 1; }
  if identity(true) { score = score + 1; }

  var opt = wrap_some(99);
  if opt.is_some { score = score + 1; }

  var none_opt: Option[Int] = None;
  if unwrap_or(none_opt, -1) == -1 { score = score + 1; }

  var ok_opt = unwrap_or(opt, 0);
  if ok_opt == 99 { score = score + 1; }

  var res_int = result_ok(42);
  match res_int {
    Ok(_) => { score = score + 1; }
    Err(_) => {}
  }

  var res_err = result_err("fail");
  match res_err {
    Ok(_) => {}
    Err(_) => { score = score + 1; }
  }

  return score;
}

fn run_errors() -> Int {
  var score = 0;
  var mod_ok = safe_mod(10, 3);
  match mod_ok {
    Ok(_) => { score = score + 1; }
    Err(_) => {}
  }

  var mod_err = safe_mod(10, 0);
  match mod_err {
    Ok(_) => {}
    Err(_) => { score = score + 1; }
  }

  var div_ok = safe_div(10, 2);
  match div_ok {
    Ok(_) => { score = score + 1; }
    Err(_) => {}
  }

  var div_err = safe_div(10, 0);
  match div_err {
    Ok(_) => {}
    Err(_) => { score = score + 1; }
  }

  var sub_ok = safe_sub(10, 3);
  match sub_ok {
    Ok(_) => { score = score + 1; }
    Err(_) => {}
  }

  var sub_err = safe_sub(3, 10);
  match sub_err {
    Ok(_) => {}
    Err(_) => { score = score + 1; }
  }

  return score;
}

fn run_contracts() -> Int {
  var c = make_counter(5, 0, 10);
  var score = 0;
  if c.value == 5 { score = score + 1; }
  if c.lo == 0 { score = score + 1; }
  if c.hi == 10 { score = score + 1; }

  var c2 = counter_inc(&c);
  if c2.value == 6 { score = score + 1; }

  var c3 = counter_dec(&c2);
  if c3.value == 5 { score = score + 1; }

  var d = safe_sqrt(9.0);
  if d == 9.0 { score = score + 1; }

  return score;
}

fn main() -> Int {
  var total = 0;
  total = total + run_math();
  total = total + run_predicates();
  total = total + run_control();
  total = total + run_structures();
  total = total + run_generics();
  total = total + run_errors();
  total = total + run_contracts();
  return total;
}
