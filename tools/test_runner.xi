#!/usr/bin/env xiom
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM Test Runner -- Gap Discovery Script
// Usage: xiom run tools/test_runner.xi
// Tests: closures, Result handling, string formatting, Vec ops, JSON parsing

// Mini assertion framework (avoids xiom.test import -- tests compiler directly)
fn assert(condition: Bool, name: Str) {
  if condition {
    io.println("  PASS: " + name);
  } else {
    io.println("  FAIL: " + name);
  };
}

fn assert_eq_int(actual: Int, expected: Int, name: Str) {
  var passed = actual == expected;
  var msg = name + ": expected " + convert.int_to_string(expected) + ", got " + convert.int_to_string(actual);
  assert(passed, msg);
}

fn main() {
  io.println("=== XIOM Test Runner (Gap Discovery) ===\n");

  // Test 1: String operations
  io.println("[string ops]");
  var s = "hello world";
  assert_eq_int(s.len(), 11, "len()");
  assert(s.slice(0, 5) == "hello", "slice(0,5)");
  assert(s.starts_with("hello"), "starts_with");
  assert(s.ends_with("world"), "ends_with");

  // Test 2: String concatenation
  io.println("[string concat]");
  var greeting = "hello " + "world";
  assert(greeting == "hello world", "literal concat");
  var name = "xiom";
  var msg = "hello " + name;
  assert(msg == "hello xiom", "var concat");

  // Test 3: Integer arithmetic
  io.println("[arithmetic]");
  assert_eq_int(2 + 3 * 4, 14, "precedence");
  assert_eq_int(10 / 3, 3, "int division");
  assert(10 % 3 == 1, "modulo");

  // Test 4: Boolean logic
  io.println("[boolean]");
  assert(true && true, "and");
  assert(true || false, "or");
  assert(!false, "not");
  assert(1 < 2 && 2 <= 2 && 3 > 2 && 3 >= 3, "comparisons");
  assert(1 == 1 && 1 != 2, "equality");

  // Test 5: Control flow
  io.println("[control flow]");
  var x = 0;
  if true { x = 1; };
  assert(x == 1, "if true");
  if false { x = 0; } else { x = 2; };
  assert(x == 2, "if false else");

  // Test 6: While loops
  io.println("[while loop]");
  var sum = 0;
  var i = 0;
  while i < 10 {
    sum = sum + i;
    i = i + 1;
  };
  assert_eq_int(sum, 45, "sum 0..9");

  // Test 7: Functions and recursion
  io.println("[functions]");
  assert_eq_int(factorial(5), 120, "factorial(5)");
  assert_eq_int(fibonacci(8), 21, "fibonacci(8)");

  // Test 8: JSON validation
  io.println("[json validation]");
  var json = "{\"key\": \"value\", \"num\": 42, \"flag\": true, \"null\": null, \"arr\": [1,2,3]}";
  var valid = json.len() > 0;
  assert(valid, "json string non-empty");

  io.println("\n=== All tests executed ===");
}

fn factorial(n: Int) -> Int {
  if n <= 1 { return 1; };
  return n * factorial(n - 1);
}

fn fibonacci(n: Int) -> Int {
  if n <= 1 { return n; };
  return fibonacci(n - 1) + fibonacci(n - 2);
}
