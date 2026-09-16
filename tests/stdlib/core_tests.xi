// XIOM -- Core Library Conformance Tests
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module core_tests
use xiom.test;
use xiom.core;
use xiom.cmp;
use xiom.convert;
use xiom.error;
use xiom.num;
use xiom.char;
use xiom.fmt;
use xiom.hash;
use xiom.iter;

fn test_option_is_some_true() -> TestResult { let v = Some(42); return assert(v.is_some(), "Option[Int]::is_some true"); }
fn test_option_is_some_false() -> TestResult { let v: Option[Int] = None; return assert(!v.is_some(), "Option[Int]::is_some false"); }
fn test_option_is_none_true() -> TestResult { let v: Option[Int] = None; return assert(v.is_none(), "Option[Int]::is_none true"); }
fn test_option_is_none_false() -> TestResult { let v = Some(42); return assert(!v.is_none(), "Option[Int]::is_none false"); }
fn test_option_unwrap_some() -> TestResult { if Some(42).unwrap() == 42 { return assert(true, "Option::unwrap some"); } return assert(false, "Option::unwrap some"); }
fn test_option_unwrap_or() -> TestResult { if None.unwrap_or(99) == 99 { return assert(true, "Option::unwrap_or"); } return assert(false, "Option::unwrap_or"); }
fn test_option_unwrap_or_some() -> TestResult { if Some(42).unwrap_or(99) == 42 { return assert(true, "Option::unwrap_or some"); } return assert(false, "Option::unwrap_or some"); }

fn test_result_ok_is_ok() -> TestResult { let r: Result[Int, Str] = Ok(42); return assert(r.is_ok(), "Result::is_ok true"); }
fn test_result_err_is_ok() -> TestResult { let r: Result[Int, Str] = Err("fail"); return assert(!r.is_ok(), "Result::is_ok false"); }
fn test_result_ok_is_err() -> TestResult { let r: Result[Int, Str] = Ok(42); return assert(!r.is_err(), "Result::is_err false"); }
fn test_result_err_is_err() -> TestResult { let r: Result[Int, Str] = Err("fail"); return assert(r.is_err(), "Result::is_err true"); }
fn test_result_unwrap_ok() -> TestResult { if Ok(42).unwrap() == 42 { return assert(true, "Result::unwrap ok"); } return assert(false, "Result::unwrap ok"); }
fn test_result_unwrap_or_ok() -> TestResult { if Ok::<Int, Str>(42).unwrap_or(99) == 42 { return assert(true, "Result::unwrap_or ok"); } return assert(false, "Result::unwrap_or ok"); }
fn test_result_unwrap_or_err() -> TestResult { if (Err::<Int, Str>("fail")).unwrap_or(99) == 99 { return assert(true, "Result::unwrap_or err"); } return assert(false, "Result::unwrap_or err"); }

fn test_int_add() -> TestResult { if 2 + 2 == 4 { return assert(true, "Int add"); } return assert(false, "Int add"); }
fn test_int_sub() -> TestResult { if 10 - 3 == 7 { return assert(true, "Int sub"); } return assert(false, "Int sub"); }
fn test_int_mul() -> TestResult { if 5 * 6 == 30 { return assert(true, "Int mul"); } return assert(false, "Int mul"); }
fn test_int_div() -> TestResult { if 20 / 4 == 5 { return assert(true, "Int div"); } return assert(false, "Int div"); }
fn test_int_rem() -> TestResult { if 17 % 5 == 2 { return assert(true, "Int rem"); } return assert(false, "Int rem"); }
fn test_int_neg() -> TestResult { if -(42) == -42 { return assert(true, "Int neg"); } return assert(false, "Int neg"); }
fn test_int_cmp_eq() -> TestResult { if 42 == 42 { return assert(true, "Int =="); } return assert(false, "Int =="); }
fn test_int_cmp_ne() -> TestResult { if 42 != 43 { return assert(true, "Int !="); } return assert(false, "Int !="); }
fn test_int_cmp_lt() -> TestResult { if 10 < 20 { return assert(true, "Int <"); } return assert(false, "Int <"); }
fn test_int_cmp_gt() -> TestResult { if 20 > 10 { return assert(true, "Int >"); } return assert(false, "Int >"); }
fn test_int_cmp_le() -> TestResult { if 10 <= 10 && 10 <= 20 { return assert(true, "Int <="); } return assert(false, "Int <="); }
fn test_int_cmp_ge() -> TestResult { if 20 >= 10 && 20 >= 20 { return assert(true, "Int >="); } return assert(false, "Int >="); }
fn test_int_zero() -> TestResult { if 0 + 0 == 0 && 0 * 100 == 0 { return assert(true, "Int zero"); } return assert(false, "Int zero"); }
fn test_int_large() -> TestResult { if 999999 + 1 == 1000000 { return assert(true, "Int large"); } return assert(false, "Int large"); }

fn test_float_add() -> TestResult { if 1.5 + 2.5 == 4.0 { return assert(true, "Float64 add"); } return assert(false, "Float64 add"); }
fn test_float_mul() -> TestResult { if 2.0 * 3.0 == 6.0 { return assert(true, "Float64 mul"); } return assert(false, "Float64 mul"); }
fn test_float_div() -> TestResult { if 10.0 / 2.0 == 5.0 { return assert(true, "Float64 div"); } return assert(false, "Float64 div"); }
fn test_float_zero() -> TestResult { if 0.0 + 0.0 == 0.0 { return assert(true, "Float64 zero"); } return assert(false, "Float64 zero"); }
fn test_float_neg() -> TestResult { if -(3.14) < 0.0 { return assert(true, "Float64 neg"); } return assert(false, "Float64 neg"); }

fn test_bool_true() -> TestResult { if true { return assert(true, "Bool true"); } return assert(false, "Bool true"); }
fn test_bool_false() -> TestResult { if !false { return assert(true, "Bool false"); } return assert(false, "Bool false"); }
fn test_bool_and() -> TestResult { if true && true && !(true && false) { return assert(true, "Bool &&"); } return assert(false, "Bool &&"); }
fn test_bool_or() -> TestResult { if true || false && !(false || false) { return assert(true, "Bool ||"); } return assert(false, "Bool ||"); }
fn test_bool_not() -> TestResult { if !false && !!true { return assert(true, "Bool !"); } return assert(false, "Bool !"); }

fn test_if_chain() -> TestResult { var x = 0; if 1==1 { x=1; } elif 2==3 { x=2; } else { x=3; } if x==1 { return assert(true, "if/elif/else"); } return assert(false, "if/elif/else"); }
fn test_while_count() -> TestResult { var i=0; while i<10 { i=i+1; } if i==10 { return assert(true, "while count"); } return assert(false, "while count"); }
fn test_nested_if() -> TestResult { if true { if true { return assert(true, "nested if"); } } return assert(false, "nested if"); }

fn test_let_inference() -> TestResult { let x = 42; if x == 42 { return assert(true, "let inference"); } return assert(false, "let inference"); }
fn test_var_mutation() -> TestResult { var x = 0; x = x + 1; x = x * 2; if x == 2 { return assert(true, "var mutation"); } return assert(false, "var mutation"); }

// === Option additional tests ===
fn test_option_unwrap_or_else() -> TestResult { if None.unwrap_or_else(|| 99) == 99 { return assert(true, "Option::unwrap_or_else"); } return assert(false, "Option::unwrap_or_else"); }
fn test_option_unwrap_or_else_some() -> TestResult { if Some(42).unwrap_or_else(|| 99) == 42 { return assert(true, "Option::unwrap_or_else some"); } return assert(false, "Option::unwrap_or_else some"); }
fn test_option_map_some() -> TestResult { if Some(42).map(|x| x * 2) == Some(84) { return assert(true, "Option::map some"); } return assert(false, "Option::map some"); }
fn test_option_map_none() -> TestResult { let v: Option[Int] = None; if v.map(|x| x * 2).is_none() { return assert(true, "Option::map none"); } return assert(false, "Option::map none"); }
fn test_option_and_then_some() -> TestResult { if Some(42).and_then(|x| Some(x * 2)) == Some(84) { return assert(true, "Option::and_then some"); } return assert(false, "Option::and_then some"); }
fn test_option_and_then_none() -> TestResult { let v: Option[Int] = None; if v.and_then(|x| Some(x * 2)).is_none() { return assert(true, "Option::and_then none"); } return assert(false, "Option::and_then none"); }
fn test_option_filter_true() -> TestResult { if Some(42).filter(|x| x > 0) == Some(42) { return assert(true, "Option::filter true"); } return assert(false, "Option::filter true"); }
fn test_option_filter_false() -> TestResult { if Some(42).filter(|x| x < 0).is_none() { return assert(true, "Option::filter false"); } return assert(false, "Option::filter false"); }
fn test_option_is_some_and_true() -> TestResult { if Some(42).is_some_and(|x| x > 0) { return assert(true, "Option::is_some_and true"); } return assert(false, "Option::is_some_and true"); }
fn test_option_is_some_and_false() -> TestResult { if !Some(42).is_some_and(|x| x < 0) { return assert(true, "Option::is_some_and false"); } return assert(false, "Option::is_some_and false"); }

// === Result additional tests ===
fn test_result_unwrap_or_else() -> TestResult { if Err::<Int, Str>("fail").unwrap_or_else(|e| e.len()) == 4 { return assert(true, "Result::unwrap_or_else"); } return assert(false, "Result::unwrap_or_else"); }
fn test_result_map_ok() -> TestResult { if Ok(42).map(|x| x * 2) == Ok(84) { return assert(true, "Result::map ok"); } return assert(false, "Result::map ok"); }
fn test_result_map_err_passthrough() -> TestResult { let r: Result[Int, Str] = Err("fail"); if r.map(|x| x * 2).is_err() { return assert(true, "Result::map err passthrough"); } return assert(false, "Result::map err passthrough"); }
fn test_result_map_err() -> TestResult { if Err::<Int, Str>("fail").map_err(|e| "wrapped") == Err("wrapped") { return assert(true, "Result::map_err"); } return assert(false, "Result::map_err"); }
fn test_result_and_then_ok() -> TestResult { if Ok(21).and_then(|x| Ok(x * 2)) == Ok(42) { return assert(true, "Result::and_then ok"); } return assert(false, "Result::and_then ok"); }
fn test_result_and_then_err() -> TestResult { let r: Result[Int, Str] = Err("fail"); if r.and_then(|x| Ok(x * 2)).is_err() { return assert(true, "Result::and_then err"); } return assert(false, "Result::and_then err"); }
fn test_result_expect_ok() -> TestResult { if Ok(42).expect("should not fail") == 42 { return assert(true, "Result::expect ok"); } return assert(false, "Result::expect ok"); }
fn test_result_is_ok_and_true() -> TestResult { if Ok(42).is_ok_and(|x| x > 0) { return assert(true, "Result::is_ok_and true"); } return assert(false, "Result::is_ok_and true"); }
fn test_result_is_ok_and_false() -> TestResult { if !Ok(42).is_ok_and(|x| x < 0) { return assert(true, "Result::is_ok_and false"); } return assert(false, "Result::is_ok_and false"); }

// === Box tests ===
fn test_box_new_get() -> TestResult { let b = core.Box.new(42); if b.get() == 42 { return assert(true, "Box::new/get"); } return assert(false, "Box::new/get"); }

// === BinaryHeap tests ===
fn test_binary_heap_new_empty() -> TestResult { let h = core.BinaryHeap[Int].new(); if h.is_empty() { return assert(true, "BinaryHeap::new is_empty"); } return assert(false, "BinaryHeap::new is_empty"); }
fn test_binary_heap_push_len() -> TestResult { var h = core.BinaryHeap[Int].new(); h.push(1); h.push(5); h.push(3); if h.len() == 3 { return assert(true, "BinaryHeap::push/len"); } return assert(false, "BinaryHeap::push/len"); }
fn test_binary_heap_pop_max() -> TestResult { var h = core.BinaryHeap[Int].new(); h.push(1); h.push(5); h.push(3); if h.pop() == 5 { return assert(true, "BinaryHeap::pop max"); } return assert(false, "BinaryHeap::pop max"); }
fn test_binary_heap_peek() -> TestResult { var h = core.BinaryHeap[Int].new(); h.push(10); h.push(20); h.push(5); if h.peek() == 20 && h.len() == 3 { return assert(true, "BinaryHeap::peek"); } return assert(false, "BinaryHeap::peek"); }
fn test_binary_heap_order() -> TestResult { var h = core.BinaryHeap[Int].new(); h.push(1); h.push(5); h.push(3); if h.pop() == 5 && h.pop() == 3 && h.pop() == 1 && h.is_empty() { return assert(true, "BinaryHeap::order"); } return assert(false, "BinaryHeap::order"); }

// === Vec predicates ===
fn test_is_sorted_true() -> TestResult { let v = [1, 2, 3, 4, 5]; if core.is_sorted(v) { return assert(true, "core::is_sorted true"); } return assert(false, "core::is_sorted true"); }
fn test_is_sorted_false() -> TestResult { let v = [3, 1, 4, 2]; if !core.is_sorted(v) { return assert(true, "core::is_sorted false"); } return assert(false, "core::is_sorted false"); }
fn test_all_predicate() -> TestResult { let v = [2, 4, 6, 8]; if core.all(v, |x| x % 2 == 0) { return assert(true, "core::all"); } return assert(false, "core::all"); }
fn test_all_predicate_false() -> TestResult { let v = [2, 4, 5, 8]; if !core.all(v, |x| x % 2 == 0) { return assert(true, "core::all false"); } return assert(false, "core::all false"); }
fn test_none_predicate() -> TestResult { let v = [1, 3, 5, 7]; if core.none(v, |x| x % 2 == 0) { return assert(true, "core::none"); } return assert(false, "core::none"); }
fn test_contains_true() -> TestResult { let v = [1, 2, 3, 4, 5]; if core.contains(v, 3) { return assert(true, "core::contains true"); } return assert(false, "core::contains true"); }
fn test_contains_false() -> TestResult { let v = [1, 2, 3, 4, 5]; if !core.contains(v, 99) { return assert(true, "core::contains false"); } return assert(false, "core::contains false"); }

// === cmp tests ===
fn test_cmp_max() -> TestResult { if cmp.max(10, 20) == 20 { return assert(true, "cmp::max"); } return assert(false, "cmp::max"); }
fn test_cmp_min() -> TestResult { if cmp.min(10, 20) == 10 { return assert(true, "cmp::min"); } return assert(false, "cmp::min"); }
fn test_cmp_clamp_inside() -> TestResult { if cmp.clamp(5, 0, 10) == 5 { return assert(true, "cmp::clamp inside"); } return assert(false, "cmp::clamp inside"); }
fn test_cmp_clamp_below() -> TestResult { if cmp.clamp(-5, 0, 10) == 0 { return assert(true, "cmp::clamp below"); } return assert(false, "cmp::clamp below"); }
fn test_cmp_clamp_above() -> TestResult { if cmp.clamp(15, 0, 10) == 10 { return assert(true, "cmp::clamp above"); } return assert(false, "cmp::clamp above"); }

// === convert tests ===
fn test_convert_int_to_float() -> TestResult { if convert.int_to_float(42) == 42.0 { return assert(true, "convert::int_to_float"); } return assert(false, "convert::int_to_float"); }
fn test_convert_float_to_int() -> TestResult { if convert.float_to_int(3.7) == 3 { return assert(true, "convert::float_to_int truncation"); } return assert(false, "convert::float_to_int truncation"); }
fn test_convert_float_to_int_neg() -> TestResult { if convert.float_to_int(-3.7) == -3 { return assert(true, "convert::float_to_int negative"); } return assert(false, "convert::float_to_int negative"); }
fn test_convert_int_to_string() -> TestResult { if convert.int_to_string(42) == "42" { return assert(true, "convert::int_to_string"); } return assert(false, "convert::int_to_string"); }
fn test_convert_int_to_string_neg() -> TestResult { if convert.int_to_string(-7) == "-7" { return assert(true, "convert::int_to_string neg"); } return assert(false, "convert::int_to_string neg"); }
fn test_convert_float_to_string() -> TestResult { let s = convert.float_to_string(3.14); if s.len() > 0 { return assert(true, "convert::float_to_string"); } return assert(false, "convert::float_to_string"); }
fn test_convert_bool_to_string_true() -> TestResult { if convert.bool_to_string(true) == "true" { return assert(true, "convert::bool_to_string true"); } return assert(false, "convert::bool_to_string true"); }
fn test_convert_bool_to_string_false() -> TestResult { if convert.bool_to_string(false) == "false" { return assert(true, "convert::bool_to_string false"); } return assert(false, "convert::bool_to_string false"); }

// === error tests ===
fn test_error_new_and_message() -> TestResult { let e = error.new("test failure"); if e.to_string().len() > 0 { return assert(true, "Error::new/message"); } return assert(false, "Error::new/message"); }
fn test_error_context() -> TestResult { let e = error.new("inner error"); let ctx = e.context("outer context"); if ctx.to_string().len() > e.to_string().len() { return assert(true, "Error::context wrapping"); } return assert(false, "Error::context wrapping"); }
fn test_error_chain_source() -> TestResult { let inner = error.new("root cause"); let outer = inner.context("wrapper"); if outer.source().is_some() { return assert(true, "Error::chain source"); } return assert(false, "Error::chain source"); }
fn test_error_backtrace_exists() -> TestResult { let e = error.new("test failure"); let bt = e.backtrace(); if bt.len() > 0 { return assert(true, "Error::backtrace exists"); } return assert(false, "Error::backtrace exists"); }

// === num tests ===
fn test_num_gcd() -> TestResult { if num.gcd(12, 8) == 4 { return assert(true, "num::gcd"); } return assert(false, "num::gcd"); }
fn test_num_gcd_coprime() -> TestResult { if num.gcd(7, 13) == 1 { return assert(true, "num::gcd coprime"); } return assert(false, "num::gcd coprime"); }
fn test_num_lcm() -> TestResult { if num.lcm(12, 8) == 24 { return assert(true, "num::lcm"); } return assert(false, "num::lcm"); }
fn test_num_is_power_of_two() -> TestResult { if num.is_power_of_two(64) && !num.is_power_of_two(100) { return assert(true, "num::is_power_of_two"); } return assert(false, "num::is_power_of_two"); }
fn test_num_checked_add_normal() -> TestResult { if num.checked_add(10, 20) == Some(30) { return assert(true, "num::checked_add normal"); } return assert(false, "num::checked_add normal"); }
fn test_num_checked_add_overflow() -> TestResult { if num.checked_add(9223372036854775807, 1).is_none() { return assert(true, "num::checked_add overflow"); } return assert(false, "num::checked_add overflow"); }
fn test_num_saturating_add_normal() -> TestResult { if num.saturating_add(10, 20) == 30 { return assert(true, "num::saturating_add normal"); } return assert(false, "num::saturating_add normal"); }
fn test_num_saturating_add_overflow() -> TestResult { if num.saturating_add(9223372036854775807, 1) == 9223372036854775807 { return assert(true, "num::saturating_add overflow"); } return assert(false, "num::saturating_add overflow"); }
fn test_num_wrapping_add() -> TestResult { if num.wrapping_add(10, 20) == 30 { return assert(true, "num::wrapping_add normal"); } return assert(false, "num::wrapping_add normal"); }

// === char tests ===
fn test_char_is_digit() -> TestResult { if char.is_digit('5') && !char.is_digit('a') { return assert(true, "char::is_digit"); } return assert(false, "char::is_digit"); }
fn test_char_is_alpha() -> TestResult { if char.is_alpha('a') && char.is_alpha('Z') && !char.is_alpha('5') { return assert(true, "char::is_alpha"); } return assert(false, "char::is_alpha"); }
fn test_char_is_alphanumeric() -> TestResult { if char.is_alphanumeric('a') && char.is_alphanumeric('9') && !char.is_alphanumeric('!') { return assert(true, "char::is_alphanumeric"); } return assert(false, "char::is_alphanumeric"); }
fn test_char_is_whitespace() -> TestResult { if char.is_whitespace(' ') && !char.is_whitespace('x') { return assert(true, "char::is_whitespace"); } return assert(false, "char::is_whitespace"); }
fn test_char_to_upper() -> TestResult { if char.to_upper('a') == 'A' { return assert(true, "char::to_upper"); } return assert(false, "char::to_upper"); }
fn test_char_to_lower() -> TestResult { if char.to_lower('Z') == 'z' { return assert(true, "char::to_lower"); } return assert(false, "char::to_lower"); }
fn test_char_len_utf8() -> TestResult { if char.len_utf8('A') == 1 { return assert(true, "char::len_utf8 ascii"); } return assert(false, "char::len_utf8 ascii"); }

// === fmt tests ===
fn test_fmt_write_str() -> TestResult { var f = fmt.Formatter.new(); f.write_str("hello"); if f.finish() == "hello" { return assert(true, "fmt::write_str/finish"); } return assert(false, "fmt::write_str/finish"); }
fn test_fmt_write_int() -> TestResult { var f = fmt.Formatter.new(); f.write_int(42); if f.finish() == "42" { return assert(true, "fmt::write_int"); } return assert(false, "fmt::write_int"); }
fn test_fmt_write_multiple() -> TestResult { var f = fmt.Formatter.new(); f.write_str("answer="); f.write_int(42); if f.finish() == "answer=42" { return assert(true, "fmt::write multiple"); } return assert(false, "fmt::write multiple"); }
fn test_fmt_to_str_int() -> TestResult { if fmt.to_str(42) == "42" { return assert(true, "fmt::to_str Int"); } return assert(false, "fmt::to_str Int"); }
fn test_fmt_to_str_float() -> TestResult { let s = fmt.to_str(3.14); if s.len() > 0 { return assert(true, "fmt::to_str Float64"); } return assert(false, "fmt::to_str Float64"); }
fn test_fmt_to_str_bool() -> TestResult { if fmt.to_str(true) == "true" && fmt.to_str(false) == "false" { return assert(true, "fmt::to_str Bool"); } return assert(false, "fmt::to_str Bool"); }

// === hash tests ===
fn test_hash_int() -> TestResult { let h = hash.of(42); if h > 0 { return assert(true, "hash::of Int"); } return assert(false, "hash::of Int"); }
fn test_hash_str() -> TestResult { let h = hash.of("hello"); if h > 0 { return assert(true, "hash::of Str"); } return assert(false, "hash::of Str"); }
fn test_hash_bool() -> TestResult { if hash.of(true) != hash.of(false) { return assert(true, "hash::of Bool"); } return assert(false, "hash::of Bool"); }
fn test_hash_equality_int() -> TestResult { if hash.of(42) == hash.of(42) { return assert(true, "hash::equality Int"); } return assert(false, "hash::equality Int"); }
fn test_hash_equality_str() -> TestResult { if hash.of("xiom") == hash.of("xiom") { return assert(true, "hash::equality Str"); } return assert(false, "hash::equality Str"); }

// === iter tests ===
fn test_iter_range_collect() -> TestResult { let v = iter.range(0, 5).collect(); if v == [0, 1, 2, 3, 4] { return assert(true, "iter::range collect"); } return assert(false, "iter::range collect"); }
fn test_iter_map() -> TestResult { let v = iter.range(0, 3).map(|x| x * 2).collect(); if v == [0, 2, 4] { return assert(true, "iter::map"); } return assert(false, "iter::map"); }
fn test_iter_filter() -> TestResult { let v = iter.range(0, 10).filter(|x| x % 2 == 0).collect(); if v == [0, 2, 4, 6, 8] { return assert(true, "iter::filter"); } return assert(false, "iter::filter"); }
fn test_iter_fold() -> TestResult { if iter.range(1, 5).fold(0, |acc, x| acc + x) == 10 { return assert(true, "iter::fold"); } return assert(false, "iter::fold"); }
fn test_iter_sum() -> TestResult { if iter.range(1, 6).sum() == 15 { return assert(true, "iter::sum"); } return assert(false, "iter::sum"); }
fn test_iter_product() -> TestResult { if iter.range(1, 5).product() == 24 { return assert(true, "iter::product"); } return assert(false, "iter::product"); }
fn test_iter_take() -> TestResult { let v = iter.range(0, 10).take(3).collect(); if v == [0, 1, 2] { return assert(true, "iter::take"); } return assert(false, "iter::take"); }
fn test_iter_skip() -> TestResult { let v = iter.range(0, 5).skip(2).collect(); if v == [2, 3, 4] { return assert(true, "iter::skip"); } return assert(false, "iter::skip"); }
fn test_iter_find() -> TestResult { if iter.range(0, 10).find(|x| x > 5) == Some(6) { return assert(true, "iter::find"); } return assert(false, "iter::find"); }
fn test_iter_find_none() -> TestResult { if iter.range(0, 5).find(|x| x > 10).is_none() { return assert(true, "iter::find none"); } return assert(false, "iter::find none"); }
fn test_iter_all() -> TestResult { if iter.range(0, 10).all(|x| x >= 0) { return assert(true, "iter::all"); } return assert(false, "iter::all"); }
fn test_iter_any() -> TestResult { if iter.range(0, 10).any(|x| x == 5) { return assert(true, "iter::any"); } return assert(false, "iter::any"); }

fn main() -> Int {
  var tests: Vec[fn() -> TestResult] = [
    test_option_is_some_true, test_option_is_some_false, test_option_is_none_true, test_option_is_none_false,
    test_option_unwrap_some, test_option_unwrap_or, test_option_unwrap_or_some,
    test_option_unwrap_or_else, test_option_unwrap_or_else_some,
    test_option_map_some, test_option_map_none,
    test_option_and_then_some, test_option_and_then_none,
    test_option_filter_true, test_option_filter_false,
    test_option_is_some_and_true, test_option_is_some_and_false,
    test_result_ok_is_ok, test_result_err_is_ok, test_result_ok_is_err, test_result_err_is_err,
    test_result_unwrap_ok, test_result_unwrap_or_ok, test_result_unwrap_or_err,
    test_result_unwrap_or_else,
    test_result_map_ok, test_result_map_err_passthrough, test_result_map_err,
    test_result_and_then_ok, test_result_and_then_err,
    test_result_expect_ok,
    test_result_is_ok_and_true, test_result_is_ok_and_false,
    test_int_add, test_int_sub, test_int_mul, test_int_div, test_int_rem, test_int_neg,
    test_int_cmp_eq, test_int_cmp_ne, test_int_cmp_lt, test_int_cmp_gt, test_int_cmp_le, test_int_cmp_ge,
    test_int_zero, test_int_large,
    test_float_add, test_float_mul, test_float_div, test_float_zero, test_float_neg,
    test_bool_true, test_bool_false, test_bool_and, test_bool_or, test_bool_not,
    test_if_chain, test_while_count, test_nested_if,
    test_let_inference, test_var_mutation,
    test_box_new_get,
    test_binary_heap_new_empty, test_binary_heap_push_len, test_binary_heap_pop_max,
    test_binary_heap_peek, test_binary_heap_order,
    test_is_sorted_true, test_is_sorted_false,
    test_all_predicate, test_all_predicate_false, test_none_predicate,
    test_contains_true, test_contains_false,
    test_cmp_max, test_cmp_min,
    test_cmp_clamp_inside, test_cmp_clamp_below, test_cmp_clamp_above,
    test_convert_int_to_float, test_convert_float_to_int, test_convert_float_to_int_neg,
    test_convert_int_to_string, test_convert_int_to_string_neg,
    test_convert_float_to_string,
    test_convert_bool_to_string_true, test_convert_bool_to_string_false,
    test_error_new_and_message, test_error_context, test_error_chain_source, test_error_backtrace_exists,
    test_num_gcd, test_num_gcd_coprime, test_num_lcm, test_num_is_power_of_two,
    test_num_checked_add_normal, test_num_checked_add_overflow,
    test_num_saturating_add_normal, test_num_saturating_add_overflow,
    test_num_wrapping_add,
    test_char_is_digit, test_char_is_alpha, test_char_is_alphanumeric, test_char_is_whitespace,
    test_char_to_upper, test_char_to_lower, test_char_len_utf8,
    test_fmt_write_str, test_fmt_write_int, test_fmt_write_multiple,
    test_fmt_to_str_int, test_fmt_to_str_float, test_fmt_to_str_bool,
    test_hash_int, test_hash_str, test_hash_bool,
    test_hash_equality_int, test_hash_equality_str,
    test_iter_range_collect, test_iter_map, test_iter_filter,
    test_iter_fold, test_iter_sum, test_iter_product,
    test_iter_take, test_iter_skip,
    test_iter_find, test_iter_find_none,
    test_iter_all, test_iter_any,
  ];
  return test.run_all(tests);
}
