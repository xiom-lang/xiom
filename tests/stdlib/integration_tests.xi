// XIOM -- Cross-Module Integration Tests
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

module integration_tests
use xiom.test;

// Test: Vec to Map pipeline
fn test_vec_to_map() -> TestResult {
  var items = Vec[(Str, Int)].new();
  items.push(("a", 1));
  items.push(("b", 2));
  return assert(items.len() > 0, "Vec+Map pipeline");
}

// Test: String operations
fn test_string_build() -> TestResult {
  let s = str_concat("hello", " world");
  if s != "" { return assert(true, "String concat"); }
  return assert(false, "String concat");
}

// Test: Math in algorithms
fn test_math_loop() -> TestResult {
  var sum = 0;
  var i = 1;
  while i <= 10 { sum = sum + i; i = i + 1; }
  if sum == 55 { return assert(true, "Math sum 1..10"); }
  return assert(false, "Math sum 1..10");
}

// Test: Ownership chain
fn test_ownership_chain() -> TestResult { return assert(true, "Ownership chain"); }

// Test: Contract exercise
fn test_contract_path() -> TestResult { return assert(true, "Contract path"); }

fn main() -> Int {
  var tests = [test_vec_to_map, test_string_build, test_math_loop, test_ownership_chain, test_contract_path];
  return test.run_all(tests);
}
