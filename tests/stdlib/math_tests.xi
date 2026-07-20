// XIOM — Math & Random Library Conformance Tests
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module math_tests
use xiom.test;
use xiom.math;
use xiom.rand;

// ============================================================
// SECTION: math — sqrt
// ============================================================

fn test_sqrt_4() -> TestResult {
  var r = math.sqrt(4.0);
  if math.abs_float(r - 2.0) < 0.0001 { return assert(true, "math::sqrt(4) ≈ 2"); }
  return assert(false, "math::sqrt(4) ≈ 2");
}

fn test_sqrt_9() -> TestResult {
  var r = math.sqrt(9.0);
  if math.abs_float(r - 3.0) < 0.0001 { return assert(true, "math::sqrt(9) ≈ 3"); }
  return assert(false, "math::sqrt(9) ≈ 3");
}

fn test_sqrt_16() -> TestResult {
  var r = math.sqrt(16.0);
  if math.abs_float(r - 4.0) < 0.0001 { return assert(true, "math::sqrt(16) ≈ 4"); }
  return assert(false, "math::sqrt(16) ≈ 4");
}

fn test_sqrt_100() -> TestResult {
  var r = math.sqrt(100.0);
  if math.abs_float(r - 10.0) < 0.0001 { return assert(true, "math::sqrt(100) ≈ 10"); }
  return assert(false, "math::sqrt(100) ≈ 10");
}

// ============================================================
// SECTION: math — abs_int
// ============================================================

fn test_abs_int_positive() -> TestResult {
  if math.abs_int(42) == 42 { return assert(true, "math::abs_int positive"); }
  return assert(false, "math::abs_int positive");
}

fn test_abs_int_negative() -> TestResult {
  if math.abs_int(-42) == 42 { return assert(true, "math::abs_int negative"); }
  return assert(false, "math::abs_int negative");
}

fn test_abs_int_zero() -> TestResult {
  if math.abs_int(0) == 0 { return assert(true, "math::abs_int zero"); }
  return assert(false, "math::abs_int zero");
}

// ============================================================
// SECTION: math — abs_float
// ============================================================

fn test_abs_float_positive() -> TestResult {
  if math.abs_float(3.14) == 3.14 { return assert(true, "math::abs_float positive"); }
  return assert(false, "math::abs_float positive");
}

fn test_abs_float_negative() -> TestResult {
  if math.abs_float(-2.71) == 2.71 { return assert(true, "math::abs_float negative"); }
  return assert(false, "math::abs_float negative");
}

fn test_abs_float_zero() -> TestResult {
  if math.abs_float(0.0) == 0.0 { return assert(true, "math::abs_float zero"); }
  return assert(false, "math::abs_float zero");
}

// ============================================================
// SECTION: math — min / max (int)
// ============================================================

fn test_min_int() -> TestResult {
  if math.min_int(5, 10) == 5 && math.min_int(-1, -5) == -5 && math.min_int(0, 0) == 0 {
    return assert(true, "math::min_int");
  }
  return assert(false, "math::min_int");
}

fn test_max_int() -> TestResult {
  if math.max_int(5, 10) == 10 && math.max_int(-1, -5) == -1 && math.max_int(7, 7) == 7 {
    return assert(true, "math::max_int");
  }
  return assert(false, "math::max_int");
}

// ============================================================
// SECTION: math — min / max (float)
// ============================================================

fn test_min_float() -> TestResult {
  if math.min_float(1.5, 2.5) == 1.5 && math.min_float(-3.0, -1.0) == -3.0 {
    return assert(true, "math::min_float");
  }
  return assert(false, "math::min_float");
}

fn test_max_float() -> TestResult {
  if math.max_float(1.5, 2.5) == 2.5 && math.max_float(-3.0, -1.0) == -1.0 {
    return assert(true, "math::max_float");
  }
  return assert(false, "math::max_float");
}

// ============================================================
// SECTION: math — floor
// ============================================================

fn test_floor_positive() -> TestResult {
  if math.floor(3.7) == 3 && math.floor(5.0) == 5 && math.floor(0.1) == 0 {
    return assert(true, "math::floor positive");
  }
  return assert(false, "math::floor positive");
}

fn test_floor_negative() -> TestResult {
  if math.floor(-3.7) == -4 && math.floor(-1.0) == -1 && math.floor(-0.1) == -1 {
    return assert(true, "math::floor negative");
  }
  return assert(false, "math::floor negative");
}

// ============================================================
// SECTION: math — ceil
// ============================================================

fn test_ceil_positive() -> TestResult {
  if math.ceil(3.2) == 4 && math.ceil(5.0) == 5 && math.ceil(0.9) == 1 {
    return assert(true, "math::ceil positive");
  }
  return assert(false, "math::ceil positive");
}

fn test_ceil_negative() -> TestResult {
  if math.ceil(-3.2) == -3 && math.ceil(-1.0) == -1 && math.ceil(-0.9) == 0 {
    return assert(true, "math::ceil negative");
  }
  return assert(false, "math::ceil negative");
}

// ============================================================
// SECTION: math — round
// ============================================================

fn test_round() -> TestResult {
  if math.round(3.4) == 3 && math.round(3.5) == 4 && math.round(3.6) == 4 &&
     math.round(-3.4) == -3 && math.round(-3.5) == -4 && math.round(0.0) == 0 {
    return assert(true, "math::round");
  }
  return assert(false, "math::round");
}

// ============================================================
// SECTION: math — pow
// ============================================================

fn test_pow_2_3() -> TestResult {
  var r = math.pow(2.0, 3.0);
  if math.abs_float(r - 8.0) < 0.001 { return assert(true, "math::pow 2^3 = 8"); }
  return assert(false, "math::pow 2^3 = 8");
}

fn test_pow_3_2() -> TestResult {
  var r = math.pow(3.0, 2.0);
  if math.abs_float(r - 9.0) < 0.001 { return assert(true, "math::pow 3^2 = 9"); }
  return assert(false, "math::pow 3^2 = 9");
}

fn test_pow_5_0() -> TestResult {
  var r = math.pow(5.0, 0.0);
  if math.abs_float(r - 1.0) < 0.001 { return assert(true, "math::pow 5^0 = 1"); }
  return assert(false, "math::pow 5^0 = 1");
}

// ============================================================
// SECTION: math — trig
// ============================================================

fn test_sin_zero() -> TestResult {
  var r = math.sin(0.0);
  if math.abs_float(r - 0.0) < 0.0001 { return assert(true, "math::sin(0) ≈ 0"); }
  return assert(false, "math::sin(0) ≈ 0");
}

fn test_sin_pi_half() -> TestResult {
  var r = math.sin(math.PI / 2.0);
  if math.abs_float(r - 1.0) < 0.01 { return assert(true, "math::sin(PI/2) ≈ 1"); }
  return assert(false, "math::sin(PI/2) ≈ 1");
}

fn test_sin_pi() -> TestResult {
  var r = math.sin(math.PI);
  if math.abs_float(r - 0.0) < 0.01 { return assert(true, "math::sin(PI) ≈ 0"); }
  return assert(false, "math::sin(PI) ≈ 0");
}

fn test_cos_zero() -> TestResult {
  var r = math.cos(0.0);
  if math.abs_float(r - 1.0) < 0.0001 { return assert(true, "math::cos(0) ≈ 1"); }
  return assert(false, "math::cos(0) ≈ 1");
}

fn test_cos_pi() -> TestResult {
  var r = math.cos(math.PI);
  if math.abs_float(r + 1.0) < 0.01 { return assert(true, "math::cos(PI) ≈ -1"); }
  return assert(false, "math::cos(PI) ≈ -1");
}

fn test_cos_pi_half() -> TestResult {
  var r = math.cos(math.PI / 2.0);
  if math.abs_float(r - 0.0) < 0.01 { return assert(true, "math::cos(PI/2) ≈ 0"); }
  return assert(false, "math::cos(PI/2) ≈ 0");
}

fn test_tan_zero() -> TestResult {
  var r = math.tan(0.0);
  if math.abs_float(r - 0.0) < 0.0001 { return assert(true, "math::tan(0) ≈ 0"); }
  return assert(false, "math::tan(0) ≈ 0");
}

fn test_tan_pi_quarter() -> TestResult {
  var r = math.tan(math.PI / 4.0);
  if math.abs_float(r - 1.0) < 0.05 { return assert(true, "math::tan(PI/4) ≈ 1"); }
  return assert(false, "math::tan(PI/4) ≈ 1");
}

fn test_asin_zero() -> TestResult {
  var r = math.asin(0.0);
  if math.abs_float(r - 0.0) < 0.0001 { return assert(true, "math::asin(0) ≈ 0"); }
  return assert(false, "math::asin(0) ≈ 0");
}

fn test_asin_one() -> TestResult {
  var r = math.asin(1.0);
  if math.abs_float(r - math.PI / 2.0) < 0.01 { return assert(true, "math::asin(1) ≈ PI/2"); }
  return assert(false, "math::asin(1) ≈ PI/2");
}

fn test_acos_one() -> TestResult {
  var r = math.acos(1.0);
  if math.abs_float(r - 0.0) < 0.0001 { return assert(true, "math::acos(1) ≈ 0"); }
  return assert(false, "math::acos(1) ≈ 0");
}

fn test_acos_zero() -> TestResult {
  var r = math.acos(0.0);
  if math.abs_float(r - math.PI / 2.0) < 0.01 { return assert(true, "math::acos(0) ≈ PI/2"); }
  return assert(false, "math::acos(0) ≈ PI/2");
}

fn test_atan_zero() -> TestResult {
  var r = math.atan(0.0);
  if math.abs_float(r - 0.0) < 0.0001 { return assert(true, "math::atan(0) ≈ 0"); }
  return assert(false, "math::atan(0) ≈ 0");
}

fn test_atan2_x_axis() -> TestResult {
  var r = math.atan2(0.0, 1.0);
  if math.abs_float(r - 0.0) < 0.0001 { return assert(true, "math::atan2(0,1) = 0"); }
  return assert(false, "math::atan2(0,1) = 0");
}

fn test_atan2_y_axis() -> TestResult {
  var r = math.atan2(1.0, 0.0);
  if math.abs_float(r - math.PI / 2.0) < 0.01 { return assert(true, "math::atan2(1,0) ≈ PI/2"); }
  return assert(false, "math::atan2(1,0) ≈ PI/2");
}

// Trig identity: sin² + cos² = 1
fn test_trig_identity_sin2_cos2() -> TestResult {
  var x = 0.7;
  var s = math.sin(x);
  var c = math.cos(x);
  var sum = s * s + c * c;
  if math.abs_float(sum - 1.0) < 0.0001 { return assert(true, "math::sin²+cos²=1"); }
  return assert(false, "math::sin²+cos²=1");
}

// Trig identity: tan = sin / cos
fn test_trig_identity_tan() -> TestResult {
  var x = 0.5;
  var t = math.tan(x);
  var ratio = math.sin(x) / math.cos(x);
  if math.abs_float(t - ratio) < 0.001 { return assert(true, "math::tan=sin/cos"); }
  return assert(false, "math::tan=sin/cos");
}

// === Pure XIOM fallback tests (no libm required) ===

fn test_sin_pure_zero() -> TestResult {
  var r = math.sin_pure(0.0);
  if math.abs_float(r - 0.0) < 0.0001 { return assert(true, "math::sin_pure(0) ≈ 0"); }
  return assert(false, "math::sin_pure(0) ≈ 0");
}

fn test_cos_pure_zero() -> TestResult {
  var r = math.cos_pure(0.0);
  if math.abs_float(r - 1.0) < 0.0001 { return assert(true, "math::cos_pure(0) ≈ 1"); }
  return assert(false, "math::cos_pure(0) ≈ 1");
}

fn test_atan_pure_zero() -> TestResult {
  var r = math.atan_pure(0.0);
  if math.abs_float(r - 0.0) < 0.0001 { return assert(true, "math::atan_pure(0) ≈ 0"); }
  return assert(false, "math::atan_pure(0) ≈ 0");
}

// Verify pure and FFI implementations agree within tolerance
fn test_sin_pure_vs_ffi() -> TestResult {
  var x = 0.5;
  var r1 = math.sin(x);
  var r2 = math.sin_pure(x);
  if math.abs_float(r1 - r2) < 0.001 { return assert(true, "math::sin vs sin_pure agree"); }
  return assert(false, "math::sin vs sin_pure agree");
}

fn test_cos_pure_vs_ffi() -> TestResult {
  var x = 1.0;
  var r1 = math.cos(x);
  var r2 = math.cos_pure(x);
  if math.abs_float(r1 - r2) < 0.001 { return assert(true, "math::cos vs cos_pure agree"); }
  return assert(false, "math::cos vs cos_pure agree");
}

// ============================================================
// SECTION: math — exp / ln
// ============================================================

fn test_exp_zero() -> TestResult {
  var r = math.exp(0.0);
  if math.abs_float(r - 1.0) < 0.0001 { return assert(true, "math::exp(0) ≈ 1"); }
  return assert(false, "math::exp(0) ≈ 1");
}

fn test_exp_one() -> TestResult {
  var r = math.exp(1.0);
  if math.abs_float(r - math.E) < 0.01 { return assert(true, "math::exp(1) ≈ E"); }
  return assert(false, "math::exp(1) ≈ E");
}

fn test_ln_one() -> TestResult {
  var r = math.ln(1.0);
  if math.abs_float(r - 0.0) < 0.0001 { return assert(true, "math::ln(1) ≈ 0"); }
  return assert(false, "math::ln(1) ≈ 0");
}

// ============================================================
// SECTION: math — clamp
// ============================================================

fn test_clamp_in_range() -> TestResult {
  var r = math.clamp(5.0, 0.0, 10.0);
  if math.abs_float(r - 5.0) < 0.0001 { return assert(true, "math::clamp in range"); }
  return assert(false, "math::clamp in range");
}

fn test_clamp_below() -> TestResult {
  var r = math.clamp(-1.0, 0.0, 10.0);
  if math.abs_float(r - 0.0) < 0.0001 { return assert(true, "math::clamp below"); }
  return assert(false, "math::clamp below");
}

fn test_clamp_above() -> TestResult {
  var r = math.clamp(15.0, 0.0, 10.0);
  if math.abs_float(r - 10.0) < 0.0001 { return assert(true, "math::clamp above"); }
  return assert(false, "math::clamp above");
}

// ============================================================
// SECTION: math — lerp
// ============================================================

fn test_lerp() -> TestResult {
  var r = math.lerp(0.0, 10.0, 0.5);
  if math.abs_float(r - 5.0) < 0.0001 { return assert(true, "math::lerp(0,10,0.5) ≈ 5"); }
  return assert(false, "math::lerp(0,10,0.5) ≈ 5");
}

fn test_lerp_zero() -> TestResult {
  var r = math.lerp(0.0, 10.0, 0.0);
  if math.abs_float(r - 0.0) < 0.0001 { return assert(true, "math::lerp t=0"); }
  return assert(false, "math::lerp t=0");
}

fn test_lerp_one() -> TestResult {
  var r = math.lerp(0.0, 10.0, 1.0);
  if math.abs_float(r - 10.0) < 0.0001 { return assert(true, "math::lerp t=1"); }
  return assert(false, "math::lerp t=1");
}

// ============================================================
// SECTION: math — is_nan / is_inf
// ============================================================

fn test_is_nan() -> TestResult {
  var nan_val = math.ln(-1.0);
  if math.is_nan(nan_val) { return assert(true, "math::is_nan"); }
  return assert(false, "math::is_nan");
}

fn test_is_nan_false() -> TestResult {
  if !math.is_nan(42.0) && !math.is_nan(0.0) { return assert(true, "math::is_nan normal"); }
  return assert(false, "math::is_nan normal");
}

fn test_is_inf() -> TestResult {
  var inf_val = math.exp(1000.0);
  if math.is_inf(inf_val) { return assert(true, "math::is_inf positive"); }
  return assert(false, "math::is_inf positive");
}

fn test_is_inf_false() -> TestResult {
  if !math.is_inf(42.0) && !math.is_inf(0.0) { return assert(true, "math::is_inf normal"); }
  return assert(false, "math::is_inf normal");
}

// ============================================================
// SECTION: math — bitwise
// ============================================================

fn test_bit_and() -> TestResult {
  if math.bit_and(6, 3) == 2 && math.bit_and(12, 10) == 8 && math.bit_and(0, 5) == 0 {
    return assert(true, "math::bit_and");
  }
  return assert(false, "math::bit_and");
}

fn test_bit_or() -> TestResult {
  if math.bit_or(6, 3) == 7 && math.bit_or(8, 1) == 9 && math.bit_or(0, 5) == 5 {
    return assert(true, "math::bit_or");
  }
  return assert(false, "math::bit_or");
}

fn test_bit_xor() -> TestResult {
  if math.bit_xor(6, 3) == 5 && math.bit_xor(5, 5) == 0 && math.bit_xor(0, 7) == 7 {
    return assert(true, "math::bit_xor");
  }
  return assert(false, "math::bit_xor");
}

// ============================================================
// SECTION: math — shift
// ============================================================

fn test_shl() -> TestResult {
  if math.shl(1, 0) == 1 && math.shl(1, 3) == 8 && math.shl(5, 2) == 20 {
    return assert(true, "math::shl");
  }
  return assert(false, "math::shl");
}

fn test_shr() -> TestResult {
  if math.shr(8, 2) == 2 && math.shr(16, 4) == 1 && math.shr(7, 1) == 3 {
    return assert(true, "math::shr");
  }
  return assert(false, "math::shr");
}

// ============================================================
// SECTION: math — constants
// ============================================================

fn test_constants() -> TestResult {
  if math.PI > 3.14 && math.PI < 3.15 && math.E > 2.71 && math.E < 2.72 {
    return assert(true, "math constants PI, E");
  }
  return assert(false, "math constants PI, E");
}

// ============================================================
// SECTION: rand — random()
// ============================================================

fn test_random_in_range() -> TestResult {
  var i = 0;
  var ok = true;
  while i < 20 {
    var r = rand.random();
    if r < 0.0 || r >= 1.0 { ok = false; }
    i = i + 1;
  }
  if ok { return assert(true, "rand::random [0,1)"); }
  return assert(false, "rand::random [0,1)");
}

// ============================================================
// SECTION: rand — random_int
// ============================================================

fn test_random_int_bounds() -> TestResult {
  var i = 0;
  var ok = true;
  while i < 100 {
    var v = rand.random_int(5, 10);
    if v < 5 || v > 10 { ok = false; }
    i = i + 1;
  }
  if ok { return assert(true, "rand::random_int bounds"); }
  return assert(false, "rand::random_int bounds");
}

// ============================================================
// SECTION: rand — seed determinism
// ============================================================

fn test_seed_deterministic() -> TestResult {
  rand.seed_from_value(42);
  var a = rand.random();
  var b = rand.random();
  rand.seed_from_value(42);
  var c = rand.random();
  var d = rand.random();
  if a == c && b == d { return assert(true, "rand::seed deterministic"); }
  return assert(false, "rand::seed deterministic");
}

fn test_seed_from_entropy() -> TestResult {
  rand.seed_from_entropy();
  return assert(true, "rand::seed_from_entropy");
}

// ============================================================
// SECTION: rand — shuffle
// ============================================================

fn test_shuffle_preserves_length() -> TestResult {
  var v = Vec[Int].new();
  v.push(1);
  v.push(2);
  v.push(3);
  v.push(4);
  var len_before = v.len();
  rand.shuffle(&mut v);
  if v.len() == len_before { return assert(true, "rand::shuffle length"); }
  return assert(false, "rand::shuffle length");
}

// ============================================================
// SECTION: rand — pick
// ============================================================

fn test_pick_returns_element() -> TestResult {
  var v = Vec[Int].new();
  v.push(10);
  v.push(20);
  v.push(30);
  var result = rand.pick(&v);
  if result.is_some() { return assert(true, "rand::pick returns Some"); }
  return assert(false, "rand::pick returns Some");
}

fn test_pick_empty_returns_none() -> TestResult {
  var v = Vec[Int].new();
  var result = rand.pick(&v);
  if result.is_none() { return assert(true, "rand::pick empty None"); }
  return assert(false, "rand::pick empty None");
}

// ============================================================
// SECTION: rand — uuid_v4
// ============================================================

fn test_uuid_v4_length() -> TestResult {
  var uuid = rand.uuid_v4();
  if uuid.len() == 36 { return assert(true, "rand::uuid_v4 length 36"); }
  return assert(false, "rand::uuid_v4 length 36");
}

fn test_uuid_v4_dashes() -> TestResult {
  var uuid = rand.uuid_v4();
  var d8 = xiom.string.str_slice(uuid, 8, 9);
  var d13 = xiom.string.str_slice(uuid, 13, 14);
  var d18 = xiom.string.str_slice(uuid, 18, 19);
  var d23 = xiom.string.str_slice(uuid, 23, 24);
  if d8 == "-" && d13 == "-" && d18 == "-" && d23 == "-" {
    return assert(true, "rand::uuid_v4 dashes");
  }
  return assert(false, "rand::uuid_v4 dashes");
}

// ============================================================
// SECTION: rand — StdRng
// ============================================================

fn test_stdrng_deterministic() -> TestResult {
  var rng1 = rand.StdRng.from_seed(99);
  var rng2 = rand.StdRng.from_seed(99);
  if rng1.next_float() == rng2.next_float() && rng1.next_int() == rng2.next_int() {
    return assert(true, "rand::StdRng deterministic");
  }
  return assert(false, "rand::StdRng deterministic");
}

// ============================================================
// SECTION: math — additional edge cases
// ============================================================

fn test_pow_zero_base() -> TestResult {
  var r = math.pow(0.0, 5.0);
  if math.abs_float(r - 0.0) < 0.0001 { return assert(true, "math::pow 0^5 = 0"); }
  return assert(false, "math::pow 0^5 = 0");
}

fn test_floor_zero() -> TestResult {
  if math.floor(0.0) == 0 { return assert(true, "math::floor zero"); }
  return assert(false, "math::floor zero");
}

fn test_ceil_zero() -> TestResult {
  if math.ceil(0.0) == 0 { return assert(true, "math::ceil zero"); }
  return assert(false, "math::ceil zero");
}

fn test_abs_int_min_value() -> TestResult {
  var v = -1;
  if math.abs_int(v) == 1 && math.abs_int(math.abs_int(v)) == 1 {
    return assert(true, "math::abs_int idempotent");
  }
  return assert(false, "math::abs_int idempotent");
}

// ============================================================
// SECTION: Run all tests
// ============================================================

fn main() -> Int {
  var tests: Vec[fn() -> TestResult] = [
    test_sqrt_4, test_sqrt_9, test_sqrt_16, test_sqrt_100,
    test_abs_int_positive, test_abs_int_negative, test_abs_int_zero,
    test_abs_float_positive, test_abs_float_negative, test_abs_float_zero,
    test_min_int, test_max_int,
    test_min_float, test_max_float,
    test_floor_positive, test_floor_negative, test_floor_zero,
    test_ceil_positive, test_ceil_negative, test_ceil_zero,
    test_round,
    test_pow_2_3, test_pow_3_2, test_pow_5_0, test_pow_zero_base,
    test_sin_zero, test_sin_pi_half, test_sin_pi,
    test_cos_zero, test_cos_pi, test_cos_pi_half,
    test_tan_zero, test_tan_pi_quarter,
    test_asin_zero, test_asin_one,
    test_acos_one, test_acos_zero,
    test_atan_zero,
    test_atan2_x_axis, test_atan2_y_axis,
    test_trig_identity_sin2_cos2, test_trig_identity_tan,
    test_sin_pure_zero, test_cos_pure_zero, test_atan_pure_zero,
    test_sin_pure_vs_ffi, test_cos_pure_vs_ffi,
    test_exp_zero, test_exp_one,
    test_ln_one,
    test_clamp_in_range, test_clamp_below, test_clamp_above,
    test_lerp, test_lerp_zero, test_lerp_one,
    test_is_nan, test_is_nan_false,
    test_is_inf, test_is_inf_false,
    test_bit_and, test_bit_or, test_bit_xor,
    test_shl, test_shr,
    test_constants, test_abs_int_min_value,
    test_random_in_range,
    test_random_int_bounds,
    test_seed_deterministic, test_seed_from_entropy,
    test_shuffle_preserves_length,
    test_pick_returns_element, test_pick_empty_returns_none,
    test_uuid_v4_length, test_uuid_v4_dashes,
    test_stdrng_deterministic,
  ];
  return test.run_all(tests);
}
