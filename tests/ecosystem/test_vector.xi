// XIOM -- Vector Database Ecosystem Compiler Hardening Test
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

module tests.ecosystem.test_vector
use xiom.collections;

pub type Vector = {
  data: Vec[Float32];
  dimension: Int;
}

pub type Neighbor = {
  id: Int;
  distance: Float32;
}

pub enum DistanceMetric {
  Cosine,
  DotProduct,
  Euclidean,
}

pub type VectorIndex = {
  vectors: Vec[Vector];
  ids: Vec[Int];
}

// ============================================================================
// Math Utilities
// ============================================================================

fn sqrt_f32(x: Float32) -> Float32 {
  if x <= 0.0 { return 0.0; }
  var guess = x * 0.5;
  var i = 0;
  while i < 12 {
    guess = (guess + x / guess) * 0.5;
    i = i + 1;
  }
  return guess;
}

fn abs_f32(x: Float32) -> Float32 {
  if x < 0.0 { return -x; }
  return x;
}

// ============================================================================
// Vector Operations
// ============================================================================

fn vector_new(dim: Int) -> Vector {
  var data = Vec[Float32].new();
  var i = 0;
  while i < dim {
    data.push(0.0);
    i = i + 1;
  }
  return Vector{ data: data, dimension: dim };
}

fn vector_set(v: &mut Vector, idx: Int, val: Float32) {
  if idx >= 0 {
    if idx < v.data.len() {
      v.data[idx] = val;
    }
  }
}

fn vector_get(v: &Vector, idx: Int) -> Float32 {
  if idx >= 0 {
    if idx < v.data.len() {
      return v.data[idx];
    }
  }
  return 0.0;
}

fn vector_dot(a: &Vector, b: &Vector) -> Float32 {
  var sum = 0.0;
  var len = a.dimension;
  if b.dimension < len { len = b.dimension; }
  var i = 0;
  while i < len {
    sum = sum + a.data[i] * b.data[i];
    i = i + 1;
  }
  return sum;
}

fn vector_magnitude(v: &Vector) -> Float32 {
  var sum_sq = 0.0;
  var i = 0;
  while i < v.dimension {
    var val = v.data[i];
    sum_sq = sum_sq + val * val;
    i = i + 1;
  }
  return sqrt_f32(sum_sq);
}

fn vector_add(a: &Vector, b: &Vector) -> Vector {
  var dim = a.dimension;
  if b.dimension < dim { dim = b.dimension; }
  var v = vector_new(dim);
  var i = 0;
  while i < dim {
    v.data[i] = a.data[i] + b.data[i];
    i = i + 1;
  }
  return v;
}

fn vector_sub(a: &Vector, b: &Vector) -> Vector {
  var dim = a.dimension;
  if b.dimension < dim { dim = b.dimension; }
  var v = vector_new(dim);
  var i = 0;
  while i < dim {
    v.data[i] = a.data[i] - b.data[i];
    i = i + 1;
  }
  return v;
}

fn vector_scale(v: &Vector, s: Float32) -> Vector {
  var result = vector_new(v.dimension);
  var i = 0;
  while i < v.dimension {
    result.data[i] = v.data[i] * s;
    i = i + 1;
  }
  return result;
}

fn vector_distance(a: &Vector, b: &Vector, metric: DistanceMetric) -> Float32 {
  match metric {
    Cosine => {
      let dot = vector_dot(a, b);
      let mag_a = vector_magnitude(a);
      let mag_b = vector_magnitude(b);
      if mag_a == 0.0 { return 1.0; }
      if mag_b == 0.0 { return 1.0; }
      var cos_sim = dot / (mag_a * mag_b);
      if cos_sim > 1.0 { cos_sim = 1.0; }
      if cos_sim < -1.0 { cos_sim = -1.0; }
      return 1.0 - cos_sim;
    }
    DotProduct => {
      return -vector_dot(a, b);
    }
    Euclidean => {
      var sum = 0.0;
      var dim = a.dimension;
      if b.dimension < dim { dim = b.dimension; }
      var i = 0;
      while i < dim {
        var diff = a.data[i] - b.data[i];
        sum = sum + diff * diff;
        i = i + 1;
      }
      return sqrt_f32(sum);
    }
  }
}

// ============================================================================
// VectorIndex Operations
// ============================================================================

fn index_new() -> VectorIndex {
  return VectorIndex{ vectors: Vec[Vector].new(), ids: Vec[Int].new() };
}

fn index_add(index: &mut VectorIndex, v: Vector, id: Int) {
  index.vectors.push(v);
  index.ids.push(id);
}

fn index_len(index: &VectorIndex) -> Int {
  return index.vectors.len();
}

fn index_search_knn(index: &VectorIndex, query: &Vector, k: Int) -> Vec[Neighbor] {
  var results = Vec[Neighbor].new();
  var i = 0;
  while i < index.vectors.len() {
    var dist = vector_distance(&index.vectors[i], query, DistanceMetric.Euclidean);
    var nb = Neighbor{ id: index.ids[i], distance: dist };
    var inserted = false;
    var j = 0;
    while j < results.len() {
      if dist < results[j].distance {
        results.insert(j, nb);
        inserted = true;
        break;
      }
      j = j + 1;
    }
    if inserted == false {
      results.push(nb);
    }
    if results.len() > k {
      results.pop();
    }
    i = i + 1;
  }
  return results;
}

// ============================================================================
// Float32 Comparison Helpers
// ============================================================================

fn f32_eq(a: Float32, b: Float32, epsilon: Float32) -> Bool {
  var diff = a - b;
  if diff < 0.0 { diff = -diff; }
  return diff < epsilon;
}

fn f32_approx(a: Float32, b: Float32) -> Bool {
  return f32_eq(a, b, 0.001);
}

// ============================================================================
// Test Functions
// ============================================================================

fn test_vector_new_dimension() -> Bool {
  var v = vector_new(5);
  if v.dimension == 5 { return true; }
  return false;
}

fn test_vector_new_zero_filled() -> Bool {
  var v = vector_new(3);
  var i = 0;
  while i < 3 {
    if v.data[i] != 0.0 { return false; }
    i = i + 1;
  }
  return true;
}

fn test_vector_set_get() -> Bool {
  var v = vector_new(3);
  vector_set(&mut v, 0, 1.5);
  vector_set(&mut v, 1, 2.5);
  vector_set(&mut v, 2, 3.5);
  if f32_approx(vector_get(&v, 0), 1.5) == false { return false; }
  if f32_approx(vector_get(&v, 1), 2.5) == false { return false; }
  if f32_approx(vector_get(&v, 2), 3.5) == false { return false; }
  return true;
}

fn test_vector_set_bounds() -> Bool {
  var v = vector_new(3);
  vector_set(&mut v, -1, 99.0);
  vector_set(&mut v, 5, 99.0);
  vector_set(&mut v, 1, 42.0);
  if f32_approx(vector_get(&v, 1), 42.0) { return true; }
  return false;
}

fn test_vector_get_bounds() -> Bool {
  var v = vector_new(2);
  var a = vector_get(&v, -1);
  var b = vector_get(&v, 5);
  if a == 0.0 && b == 0.0 { return true; }
  return false;
}

fn test_vector_dot_unit() -> Bool {
  var a = vector_new(3);
  var b = vector_new(3);
  vector_set(&mut a, 0, 1.0);
  vector_set(&mut a, 1, 0.0);
  vector_set(&mut a, 2, 0.0);
  vector_set(&mut b, 0, 0.0);
  vector_set(&mut b, 1, 1.0);
  vector_set(&mut b, 2, 0.0);
  var dot = vector_dot(&a, &b);
  if f32_approx(dot, 0.0) { return true; }
  return false;
}

fn test_vector_dot_self() -> Bool {
  var v = vector_new(3);
  vector_set(&mut v, 0, 1.0);
  vector_set(&mut v, 1, 2.0);
  vector_set(&mut v, 2, 3.0);
  var dot = vector_dot(&v, &v);
  if f32_approx(dot, 14.0) { return true; }
  return false;
}

fn test_vector_dot_parallel() -> Bool {
  var a = vector_new(2);
  var b = vector_new(2);
  vector_set(&mut a, 0, 2.0);
  vector_set(&mut a, 1, 3.0);
  vector_set(&mut b, 0, 4.0);
  vector_set(&mut b, 1, 6.0);
  var dot = vector_dot(&a, &b);
  if f32_approx(dot, 26.0) { return true; }
  return false;
}

fn test_vector_magnitude_unit() -> Bool {
  var v = vector_new(3);
  vector_set(&mut v, 0, 1.0);
  vector_set(&mut v, 1, 0.0);
  vector_set(&mut v, 2, 0.0);
  var mag = vector_magnitude(&v);
  if f32_approx(mag, 1.0) { return true; }
  return false;
}

fn test_vector_magnitude_345() -> Bool {
  var v = vector_new(3);
  vector_set(&mut v, 0, 3.0);
  vector_set(&mut v, 1, 4.0);
  vector_set(&mut v, 2, 0.0);
  var mag = vector_magnitude(&v);
  if f32_approx(mag, 5.0) { return true; }
  return false;
}

fn test_vector_magnitude_zero() -> Bool {
  var v = vector_new(5);
  var mag = vector_magnitude(&v);
  if f32_approx(mag, 0.0) { return true; }
  return false;
}

fn test_sqrt_perfect_squares() -> Bool {
  var s4 = sqrt_f32(4.0);
  var s9 = sqrt_f32(9.0);
  var s16 = sqrt_f32(16.0);
  var s100 = sqrt_f32(100.0);
  if f32_approx(s4, 2.0) == false { return false; }
  if f32_approx(s9, 3.0) == false { return false; }
  if f32_approx(s16, 4.0) == false { return false; }
  if f32_approx(s100, 10.0) == false { return false; }
  return true;
}

fn test_sqrt_non_perfect() -> Bool {
  var s2 = sqrt_f32(2.0);
  var s3 = sqrt_f32(3.0);
  var ok2 = s2 > 1.41 && s2 < 1.42;
  var ok3 = s3 > 1.73 && s3 < 1.74;
  if ok2 && ok3 { return true; }
  return false;
}

fn test_sqrt_zero_negative() -> Bool {
  var s0 = sqrt_f32(0.0);
  var s_neg = sqrt_f32(-5.0);
  if s0 == 0.0 && s_neg == 0.0 { return true; }
  return false;
}

fn test_cosine_distance_identical() -> Bool {
  var v = vector_new(3);
  vector_set(&mut v, 0, 1.0);
  vector_set(&mut v, 1, 2.0);
  vector_set(&mut v, 2, 3.0);
  var dist = vector_distance(&v, &v, DistanceMetric.Cosine);
  if f32_approx(dist, 0.0) { return true; }
  return false;
}

fn test_cosine_distance_orthogonal() -> Bool {
  var a = vector_new(2);
  var b = vector_new(2);
  vector_set(&mut a, 0, 1.0);
  vector_set(&mut a, 1, 0.0);
  vector_set(&mut b, 0, 0.0);
  vector_set(&mut b, 1, 1.0);
  var dist = vector_distance(&a, &b, DistanceMetric.Cosine);
  if f32_approx(dist, 1.0) { return true; }
  return false;
}

fn test_cosine_distance_opposite() -> Bool {
  var a = vector_new(2);
  var b = vector_new(2);
  vector_set(&mut a, 0, 1.0);
  vector_set(&mut a, 1, 0.0);
  vector_set(&mut b, 0, -1.0);
  vector_set(&mut b, 1, 0.0);
  var dist = vector_distance(&a, &b, DistanceMetric.Cosine);
  if f32_approx(dist, 2.0) { return true; }
  return false;
}

fn test_euclidean_distance_same() -> Bool {
  var v = vector_new(3);
  vector_set(&mut v, 0, 1.0);
  vector_set(&mut v, 1, 2.0);
  vector_set(&mut v, 2, 3.0);
  var dist = vector_distance(&v, &v, DistanceMetric.Euclidean);
  if f32_approx(dist, 0.0) { return true; }
  return false;
}

fn test_euclidean_distance_345() -> Bool {
  var a = vector_new(2);
  var b = vector_new(2);
  vector_set(&mut a, 0, 0.0);
  vector_set(&mut a, 1, 0.0);
  vector_set(&mut b, 0, 3.0);
  vector_set(&mut b, 1, 4.0);
  var dist = vector_distance(&a, &b, DistanceMetric.Euclidean);
  if f32_approx(dist, 5.0) { return true; }
  return false;
}

fn test_dotproduct_distance() -> Bool {
  var a = vector_new(2);
  var b = vector_new(2);
  vector_set(&mut a, 0, 2.0);
  vector_set(&mut a, 1, 3.0);
  vector_set(&mut b, 0, 4.0);
  vector_set(&mut b, 1, 5.0);
  var dist = vector_distance(&a, &b, DistanceMetric.DotProduct);
  if f32_approx(dist, -23.0) { return true; }
  return false;
}

fn test_distance_metric_enum_coverage() -> Bool {
  var a = vector_new(2);
  var b = vector_new(2);
  vector_set(&mut a, 0, 1.0);
  vector_set(&mut b, 0, 2.0);
  var d1 = vector_distance(&a, &b, DistanceMetric.Cosine);
  var d2 = vector_distance(&a, &b, DistanceMetric.Euclidean);
  var d3 = vector_distance(&a, &b, DistanceMetric.DotProduct);
  var ok = true;
  if d1 == d2 { ok = false; }
  return ok;
}

fn test_index_new_empty() -> Bool {
  var idx = index_new();
  if index_len(&idx) == 0 { return true; }
  return false;
}

fn test_index_add_and_len() -> Bool {
  var idx = index_new();
  var v1 = vector_new(2);
  var v2 = vector_new(2);
  vector_set(&mut v1, 0, 1.0);
  vector_set(&mut v2, 0, 2.0);
  index_add(&mut idx, v1, 100);
  index_add(&mut idx, v2, 200);
  if index_len(&idx) == 2 { return true; }
  return false;
}

fn test_knn_search_exact() -> Bool {
  var idx = index_new();
  var v1 = vector_new(2);
  vector_set(&mut v1, 0, 1.0);
  vector_set(&mut v1, 1, 0.0);
  var v2 = vector_new(2);
  vector_set(&mut v2, 0, 0.0);
  vector_set(&mut v2, 1, 1.0);
  var v3 = vector_new(2);
  vector_set(&mut v3, 0, 100.0);
  vector_set(&mut v3, 1, 100.0);
  index_add(&mut idx, v1, 10);
  index_add(&mut idx, v2, 20);
  index_add(&mut idx, v3, 30);
  var query = vector_new(2);
  vector_set(&mut query, 0, 1.0);
  vector_set(&mut query, 1, 0.0);
  var results = index_search_knn(&idx, &query, 2);
  if results.len() == 2 { return true; }
  return false;
}

fn test_knn_search_top1() -> Bool {
  var idx = index_new();
  var v1 = vector_new(2);
  vector_set(&mut v1, 0, 1.0);
  vector_set(&mut v1, 1, 0.0);
  var v2 = vector_new(2);
  vector_set(&mut v2, 0, 100.0);
  vector_set(&mut v2, 1, 100.0);
  index_add(&mut idx, v1, 10);
  index_add(&mut idx, v2, 20);
  var query = vector_new(2);
  vector_set(&mut query, 0, 1.0);
  vector_set(&mut query, 1, 0.0);
  var results = index_search_knn(&idx, &query, 1);
  if results.len() == 1 { return true; }
  return false;
}

fn test_knn_search_fewer_than_k() -> Bool {
  var idx = index_new();
  var v1 = vector_new(2);
  vector_set(&mut v1, 0, 1.0);
  vector_set(&mut v1, 1, 0.0);
  index_add(&mut idx, v1, 42);
  var query = vector_new(2);
  vector_set(&mut query, 0, 0.0);
  vector_set(&mut query, 1, 0.0);
  var results = index_search_knn(&idx, &query, 5);
  if results.len() == 1 { return true; }
  return false;
}

fn test_vector_add() -> Bool {
  var a = vector_new(2);
  var b = vector_new(2);
  vector_set(&mut a, 0, 1.0);
  vector_set(&mut a, 1, 2.0);
  vector_set(&mut b, 0, 3.0);
  vector_set(&mut b, 1, 4.0);
  var c = vector_add(&a, &b);
  if f32_approx(c.data[0], 4.0) && f32_approx(c.data[1], 6.0) { return true; }
  return false;
}

fn test_vector_sub() -> Bool {
  var a = vector_new(2);
  var b = vector_new(2);
  vector_set(&mut a, 0, 10.0);
  vector_set(&mut a, 1, 20.0);
  vector_set(&mut b, 0, 3.0);
  vector_set(&mut b, 1, 5.0);
  var c = vector_sub(&a, &b);
  if f32_approx(c.data[0], 7.0) && f32_approx(c.data[1], 15.0) { return true; }
  return false;
}

fn test_vector_scale() -> Bool {
  var v = vector_new(3);
  vector_set(&mut v, 0, 1.0);
  vector_set(&mut v, 1, 2.0);
  vector_set(&mut v, 2, 3.0);
  var s = vector_scale(&v, 2.0);
  if f32_approx(s.data[0], 2.0) && f32_approx(s.data[1], 4.0) && f32_approx(s.data[2], 6.0) { return true; }
  return false;
}

fn test_distance_metric_variant_discrimination() -> Bool {
  match DistanceMetric.Cosine {
    Cosine => { return true; }
    DotProduct => { return false; }
    Euclidean => { return false; }
  }
}

fn test_neighbor_struct() -> Bool {
  var nb = Neighbor{ id: 42, distance: 0.5 };
  if nb.id == 42 && f32_approx(nb.distance, 0.5) { return true; }
  return false;
}

fn test_vector_zero_dimension() -> Bool {
  var v = vector_new(0);
  if v.dimension == 0 && v.data.len() == 0 { return true; }
  return false;
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Int {
  var failed = false;
  if test_vector_new_dimension() == false { failed = true; }
  if test_vector_new_zero_filled() == false { failed = true; }
  if test_vector_set_get() == false { failed = true; }
  if test_vector_set_bounds() == false { failed = true; }
  if test_vector_get_bounds() == false { failed = true; }
  if test_vector_dot_unit() == false { failed = true; }
  if test_vector_dot_self() == false { failed = true; }
  if test_vector_dot_parallel() == false { failed = true; }
  if test_vector_magnitude_unit() == false { failed = true; }
  if test_vector_magnitude_345() == false { failed = true; }
  if test_vector_magnitude_zero() == false { failed = true; }
  if test_sqrt_perfect_squares() == false { failed = true; }
  if test_sqrt_non_perfect() == false { failed = true; }
  if test_sqrt_zero_negative() == false { failed = true; }
  if test_cosine_distance_identical() == false { failed = true; }
  if test_cosine_distance_orthogonal() == false { failed = true; }
  if test_cosine_distance_opposite() == false { failed = true; }
  if test_euclidean_distance_same() == false { failed = true; }
  if test_euclidean_distance_345() == false { failed = true; }
  if test_dotproduct_distance() == false { failed = true; }
  if test_distance_metric_enum_coverage() == false { failed = true; }
  if test_index_new_empty() == false { failed = true; }
  if test_index_add_and_len() == false { failed = true; }
  if test_knn_search_exact() == false { failed = true; }
  if test_knn_search_top1() == false { failed = true; }
  if test_knn_search_fewer_than_k() == false { failed = true; }
  if test_vector_add() == false { failed = true; }
  if test_vector_sub() == false { failed = true; }
  if test_vector_scale() == false { failed = true; }
  if test_distance_metric_variant_discrimination() == false { failed = true; }
  if test_neighbor_struct() == false { failed = true; }
  if test_vector_zero_dimension() == false { failed = true; }
  if failed { return 1; }
  return 0;
}
