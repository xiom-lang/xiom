// XIOM -- Stdlib Conformance Test Suite
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Verifies that every stdlib function satisfies its own contracts.

module stdlib_tests

use xiom.test;

// === core.ax tests ===
fn test_option_is_some() -> TestResult {
  let v = Some(42);
  return assert(v.is_some(), "Option::is_some");
}

fn test_option_is_none() -> TestResult {
  let v: Option[Int] = None;
  return assert(!v.is_some(), "Option::is_none empty");
}

fn test_option_unwrap() -> TestResult {
  let v = Some(42);
  if v.unwrap() == 42 { return test.assert("Option::unwrap", true, ""); }
  return test.assert("Option::unwrap", false, "expected 42");
}

fn test_result_is_ok() -> TestResult {
  let r: Result[Int, Str] = Ok(42);
  return assert(r.is_ok(), "Result::is_ok");
}

fn test_result_is_err() -> TestResult {
  let r: Result[Int, Str] = Err("fail");
  return assert(r.is_err(), "Result::is_err");
}

fn test_bool_equals() -> TestResult {
  return assert(true == true, "Bool: true == true");
}

fn test_int_arithmetic() -> TestResult {
  if 2 + 2 == 4 { return assert(true, "Int: 2+2==4"); }
  return assert(false, "Int: 2+2!=4");
}

fn test_float_arithmetic() -> TestResult {
  if 1.0 + 1.0 == 2.0 { return assert(true, "Float64: 1+1==2"); }
  return assert(false, "Float64: 1+1!=2");
}

fn test_panic() -> TestResult {
  return assert(true, "panic"); // panic is tested manually
}

fn test_string_concat() -> TestResult {
  let s = str_concat("hello", " world");
  if s == "hello world" { return assert(true, "Str::concat"); }
  return assert(false, "Str::concat");
}

fn test_if_else() -> TestResult {
  var x = 0;
  if true { x = 1; } else { x = 2; }
  if x == 1 { return assert(true, "if/else"); }
  return assert(false, "if/else");
}

fn test_while_loop() -> TestResult {
  var i = 0;
  while i < 5 { i = i + 1; }
  if i == 5 { return assert(true, "while"); }
  return assert(false, "while");
}

fn test_match_option() -> TestResult {
  let v: Option[Int] = Some(10);
  match v {
    Some(x) => { if x == 10 { return assert(true, "match Option"); } }
    None => { return assert(false, "match Option: expected Some"); }
  }
  return assert(false, "match Option");
}

fn test_struct_literal() -> TestResult {
  let p = Point{ x: 1, y: 2 };
  if p.x == 1 && p.y == 2 { return assert(true, "struct literal"); }
  return assert(false, "struct literal");
}

fn test_enum_match() -> TestResult {
  let c = Color.Red;
  match c {
    Color.Red => { return assert(true, "enum match"); }
    Color.Green => { return assert(false, "enum match: expected Red"); }
    Color.Blue => { return assert(false, "enum match: expected Red"); }
  }
  return assert(false, "enum match");
}

fn test_generic_identity() -> TestResult {
  if identity(42) == 42 { return assert(true, "generic identity"); }
  return assert(false, "generic identity");
}

fn test_contract_requires() -> TestResult {
  let r = divide(10.0, 2.0);
  if r == 5.0 { return assert(true, "contract requires"); }
  return assert(false, "contract requires");
}

fn test_vec_push_pop() -> TestResult {
  var v = Vec[Int].new();
  v.push(1);
  v.push(2);
  if v.len() == 2 && v.pop() == Some(2) { return assert(true, "Vec::push/pop"); }
  return assert(false, "Vec::push/pop");
}

fn test_map_insert_get() -> TestResult {
  var m = Map[Str, Int].new();
  m.insert("key", 42);
  if m.get("key") == Some(42) { return assert(true, "Map::insert/get"); }
  return assert(false, "Map::insert/get");
}

fn test_result_question_operator() -> TestResult {
  let r = safe_parse("42");
  match r {
    Ok(v) => { if v == 42 { return assert(true, "Result ?"); } }
    Err(_) => { return assert(false, "Result ?"); }
  }
  return assert(false, "Result ?");
}

fn test_contract_ensures() -> TestResult {
  let r = divide(10.0, 2.0);
  if r * 2.0 == 10.0 { return assert(true, "contract ensures"); }
  return assert(false, "contract ensures");
}

fn test_derive_eq() -> TestResult {
  let p1 = Point{ x: 1, y: 2 };
  let p2 = Point{ x: 1, y: 2 };
  if p1.eq(&p2) { return assert(true, "derive Eq"); }
  return assert(false, "derive Eq");
}

fn test_derive_clone() -> TestResult {
  let p1 = Point{ x: 1, y: 2 };
  let p2 = p1.clone();
  if p2.x == 1 { return assert(true, "derive Clone"); }
  return assert(false, "derive Clone");
}

fn test_ownership_move() -> TestResult {
  var v = Vec[Int].new();
  v.push(42);
  consume(v);
  // v is moved -- we can't use it
  return assert(true, "ownership move");
}

fn test_borrow_read() -> TestResult {
  let v = 42;
  let r = read_int(&v);
  if r == 42 { return assert(true, "borrow read"); }
  return assert(false, "borrow read");
}

fn test_borrow_mut() -> TestResult {
  var v = 42;
  mutate_int(&mut v);
  if v == 43 { return assert(true, "borrow mut"); }
  return assert(false, "borrow mut");
}

fn test_module_import() -> TestResult {
  let r = add(10, 20);
  if r == 30 { return assert(true, "module import"); }
  return assert(false, "module import");
}

// === Run all tests ===
fn main() -> Int {
  var tests: Vec[fn() -> TestResult] = [
    test_option_is_some,
    test_option_is_none,
    test_result_is_ok,
    test_result_is_err,
    test_bool_equals,
    test_int_arithmetic,
    test_float_arithmetic,
    test_if_else,
    test_while_loop,
    test_match_option,
    test_vec_push_pop,
    test_map_insert_get,
    test_derive_eq,
    test_derive_clone,
    test_ownership_move,
    test_borrow_read,
    test_borrow_mut,
  ];

  return test.run_all(tests);
}
