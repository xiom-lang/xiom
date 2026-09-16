// XIOM -- Ecosystem Test Framework Hardening Tests
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// A minimal self-contained test runner that exercises struct creation,
// Vec manipulation, method dispatch, and Bool logic.

module tests.ecosystem.test_test
use xiom.collections;

pub type TestCase = {
  name: Str;
  passed: Bool;
  message: Str;
}

pub type TestSuite = {
  name: Str;
  cases: Vec[TestCase];
}

pub type TestResults = {
  total: Int;
  passed: Int;
  failed: Int;
  failures: Vec[Str];
}

fn TestSuite.new(name: Str) -> TestSuite {
  var cases = Vec[TestCase].new();
  return TestSuite{ name: name, cases: cases };
}

fn TestSuite.add_case(ts: &mut TestSuite, name: Str, passed: Bool, message: Str) {
  var tc = TestCase{ name: name, passed: passed, message: message };
  ts.cases.push(tc);
}

fn TestSuite.run_all(ts: &TestSuite) -> TestResults {
  var total = ts.cases.len();
  var passed = 0;
  var failed = 0;
  var failures = Vec[Str].new();
  var i = 0;
  while i < total {
    if ts.cases[i].passed {
      passed = passed + 1;
    } else {
      failed = failed + 1;
      failures.push(ts.cases[i].name + ": " + ts.cases[i].message);
    }
    i = i + 1;
  }
  return TestResults{ total: total, passed: passed, failed: failed, failures: failures };
}

fn TestResults.is_success(r: &TestResults) -> Bool {
  return r.failed == 0;
}

fn TestResults.has_failures(r: &TestResults) -> Bool {
  return r.failed > 0;
}

fn assert_true(val: Bool, name: Str) -> TestCase {
  if val {
    return TestCase{ name: name, passed: true, message: "" };
  }
  return TestCase{ name: name, passed: false, message: "expected true, got false" };
}

fn assert_false(val: Bool, name: Str) -> TestCase {
  if !val {
    return TestCase{ name: name, passed: true, message: "" };
  }
  return TestCase{ name: name, passed: false, message: "expected false, got true" };
}

fn assert_eq_int(a: Int, b: Int, name: Str) -> TestCase {
  if a == b {
    return TestCase{ name: name, passed: true, message: "" };
  }
  return TestCase{ name: name, passed: false, message: "expected equality" };
}

fn assert_eq_str(a: Str, b: Str, name: Str) -> TestCase {
  if a == b {
    return TestCase{ name: name, passed: true, message: "" };
  }
  return TestCase{ name: name, passed: false, message: "expected equality" };
}

// ============================================================================
// Tests that test the test framework itself
// ============================================================================

fn test_suite_new_empty() -> Bool {
  let ts = TestSuite.new("empty");
  return ts.name == "empty" && ts.cases.len() == 0;
}

fn test_suite_add_case() -> Bool {
  var ts = TestSuite.new("suite");
  TestSuite.add_case(&mut ts, "test1", true, "");
  return ts.cases.len() == 1;
}

fn test_suite_add_multiple_cases() -> Bool {
  var ts = TestSuite.new("suite");
  TestSuite.add_case(&mut ts, "a", true, "");
  TestSuite.add_case(&mut ts, "b", true, "");
  TestSuite.add_case(&mut ts, "c", false, "fail");
  return ts.cases.len() == 3;
}

fn test_suite_case_attributes() -> Bool {
  var ts = TestSuite.new("suite");
  TestSuite.add_case(&mut ts, "check", false, "oops");
  return ts.cases[0].name == "check" && !ts.cases[0].passed && ts.cases[0].message == "oops";
}

fn test_run_all_passing() -> Bool {
  var ts = TestSuite.new("all_pass");
  TestSuite.add_case(&mut ts, "a", true, "");
  TestSuite.add_case(&mut ts, "b", true, "");
  TestSuite.add_case(&mut ts, "c", true, "");
  let results = TestSuite.run_all(&ts);
  return results.total == 3 && results.passed == 3 && results.failed == 0;
}

fn test_run_all_mixed() -> Bool {
  var ts = TestSuite.new("mixed");
  TestSuite.add_case(&mut ts, "ok1", true, "");
  TestSuite.add_case(&mut ts, "fail1", false, "err");
  TestSuite.add_case(&mut ts, "ok2", true, "");
  TestSuite.add_case(&mut ts, "fail2", false, "err2");
  let results = TestSuite.run_all(&ts);
  return results.total == 4 && results.passed == 2 && results.failed == 2 && results.failures.len() == 2;
}

fn test_is_success_true() -> Bool {
  var ts = TestSuite.new("s");
  TestSuite.add_case(&mut ts, "t", true, "");
  let r = TestSuite.run_all(&ts);
  return TestResults.is_success(&r);
}

fn test_is_success_false() -> Bool {
  var ts = TestSuite.new("s");
  TestSuite.add_case(&mut ts, "t", false, "fail");
  let r = TestSuite.run_all(&ts);
  return !TestResults.is_success(&r);
}

fn test_has_failures_true() -> Bool {
  var ts = TestSuite.new("s");
  TestSuite.add_case(&mut ts, "t", false, "fail");
  let r = TestSuite.run_all(&ts);
  return TestResults.has_failures(&r);
}

fn test_has_failures_false() -> Bool {
  var ts = TestSuite.new("s");
  TestSuite.add_case(&mut ts, "t", true, "");
  let r = TestSuite.run_all(&ts);
  return !TestResults.has_failures(&r);
}

fn test_assert_true_passing() -> Bool {
  let tc = assert_true(true, "truth");
  return tc.passed && tc.name == "truth";
}

fn test_assert_true_failing() -> Bool {
  let tc = assert_true(false, "lie");
  return !tc.passed && tc.name == "lie";
}

fn test_assert_false_passing() -> Bool {
  let tc = assert_false(false, "false");
  return tc.passed && tc.name == "false";
}

fn test_assert_false_failing() -> Bool {
  let tc = assert_false(true, "not false");
  return !tc.passed && tc.name == "not false";
}

fn test_assert_eq_int_pass() -> Bool {
  let tc = assert_eq_int(42, 42, "int_eq");
  return tc.passed && tc.name == "int_eq";
}

fn test_assert_eq_int_fail() -> Bool {
  let tc = assert_eq_int(1, 2, "int_neq");
  return !tc.passed && tc.name == "int_neq";
}

fn test_assert_eq_str_pass() -> Bool {
  let tc = assert_eq_str("hello", "hello", "str_eq");
  return tc.passed && tc.name == "str_eq";
}

fn test_assert_eq_str_fail() -> Bool {
  let tc = assert_eq_str("hello", "world", "str_neq");
  return !tc.passed && tc.name == "str_neq";
}

fn test_failures_contain_name() -> Bool {
  var ts = TestSuite.new("s");
  TestSuite.add_case(&mut ts, "alpha", true, "");
  TestSuite.add_case(&mut ts, "beta", false, "broken");
  TestSuite.add_case(&mut ts, "gamma", false, "busted");
  let r = TestSuite.run_all(&ts);
  if r.failures.len() != 2 { return false; }
  var has_beta = false;
  var has_gamma = false;
  var i = 0;
  while i < r.failures.len() {
    if r.failures[i] == "beta: broken" { has_beta = true; }
    if r.failures[i] == "gamma: busted" { has_gamma = true; }
    i = i + 1;
  }
  return has_beta && has_gamma;
}

fn test_empty_suite_results() -> Bool {
  let ts = TestSuite.new("empty");
  let r = TestSuite.run_all(&ts);
  return r.total == 0 && r.passed == 0 && r.failed == 0 && r.failures.len() == 0;
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Int {
  var passed = 0;
  var total = 0;

  total = total + 1;
  if test_suite_new_empty() { passed = passed + 1; }

  total = total + 1;
  if test_suite_add_case() { passed = passed + 1; }

  total = total + 1;
  if test_suite_add_multiple_cases() { passed = passed + 1; }

  total = total + 1;
  if test_suite_case_attributes() { passed = passed + 1; }

  total = total + 1;
  if test_run_all_passing() { passed = passed + 1; }

  total = total + 1;
  if test_run_all_mixed() { passed = passed + 1; }

  total = total + 1;
  if test_is_success_true() { passed = passed + 1; }

  total = total + 1;
  if test_is_success_false() { passed = passed + 1; }

  total = total + 1;
  if test_has_failures_true() { passed = passed + 1; }

  total = total + 1;
  if test_has_failures_false() { passed = passed + 1; }

  total = total + 1;
  if test_assert_true_passing() { passed = passed + 1; }

  total = total + 1;
  if test_assert_true_failing() { passed = passed + 1; }

  total = total + 1;
  if test_assert_false_passing() { passed = passed + 1; }

  total = total + 1;
  if test_assert_false_failing() { passed = passed + 1; }

  total = total + 1;
  if test_assert_eq_int_pass() { passed = passed + 1; }

  total = total + 1;
  if test_assert_eq_int_fail() { passed = passed + 1; }

  total = total + 1;
  if test_assert_eq_str_pass() { passed = passed + 1; }

  total = total + 1;
  if test_assert_eq_str_fail() { passed = passed + 1; }

  total = total + 1;
  if test_failures_contain_name() { passed = passed + 1; }

  total = total + 1;
  if test_empty_suite_results() { passed = passed + 1; }

  if passed == total { return 0; }
  return 1;
}
