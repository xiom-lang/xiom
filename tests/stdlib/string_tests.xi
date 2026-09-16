// XIOM -- String Library Conformance Tests
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

module string_tests
use xiom.test;
use xiom.string;

fn test_str_len_empty() -> TestResult { if xiom.string.str_len("") == 0 { return assert(true, "str_len empty"); } return assert(false, "str_len empty"); }
fn test_str_len_nonempty() -> TestResult { if xiom.string.str_len("hello") == 5 { return assert(true, "str_len non-empty"); } return assert(false, "str_len non-empty"); }

fn test_str_concat_two() -> TestResult { if xiom.string.str_concat("hello", " world") == "hello world" { return assert(true, "str_concat two"); } return assert(false, "str_concat two"); }
fn test_str_concat_empty_first() -> TestResult { if xiom.string.str_concat("", "world") == "world" { return assert(true, "str_concat empty first"); } return assert(false, "str_concat empty first"); }
fn test_str_concat_empty_second() -> TestResult { if xiom.string.str_concat("hello", "") == "hello" { return assert(true, "str_concat empty second"); } return assert(false, "str_concat empty second"); }
fn test_str_concat_empty_both() -> TestResult { if xiom.string.str_concat("", "") == "" { return assert(true, "str_concat empty both"); } return assert(false, "str_concat empty both"); }

fn test_str_slice_valid() -> TestResult { if xiom.string.str_slice("hello world", 0, 5) == "hello" { return assert(true, "str_slice valid"); } return assert(false, "str_slice valid"); }
fn test_str_slice_full() -> TestResult { if xiom.string.str_slice("hello", 0, 5) == "hello" { return assert(true, "str_slice start=0 end=len"); } return assert(false, "str_slice start=0 end=len"); }
fn test_str_slice_middle() -> TestResult { if xiom.string.str_slice("abcdef", 2, 4) == "cd" { return assert(true, "str_slice middle"); } return assert(false, "str_slice middle"); }
fn test_str_slice_empty_result() -> TestResult { if xiom.string.str_slice("hello", 2, 2) == "" { return assert(true, "str_slice empty result"); } return assert(false, "str_slice empty result"); }

fn test_str_contains_present() -> TestResult { if xiom.string.str_contains("hello world", "world") { return assert(true, "str_contains present"); } return assert(false, "str_contains present"); }
fn test_str_contains_absent() -> TestResult { if !xiom.string.str_contains("hello world", "xyz") { return assert(true, "str_contains absent"); } return assert(false, "str_contains absent"); }
fn test_str_contains_empty_substring() -> TestResult { if xiom.string.str_contains("hello", "") { return assert(true, "str_contains empty substring"); } return assert(false, "str_contains empty substring"); }

fn test_str_starts_with_present() -> TestResult { if xiom.string.str_starts_with("hello world", "hello") { return assert(true, "str_starts_with present"); } return assert(false, "str_starts_with present"); }
fn test_str_starts_with_absent() -> TestResult { if !xiom.string.str_starts_with("hello world", "world") { return assert(true, "str_starts_with absent"); } return assert(false, "str_starts_with absent"); }
fn test_str_starts_with_longer_prefix() -> TestResult { if !xiom.string.str_starts_with("hi", "hello") { return assert(true, "str_starts_with longer prefix"); } return assert(false, "str_starts_with longer prefix"); }

fn test_str_ends_with_present() -> TestResult { if xiom.string.str_ends_with("hello world", "world") { return assert(true, "str_ends_with present"); } return assert(false, "str_ends_with present"); }
fn test_str_ends_with_absent() -> TestResult { if !xiom.string.str_ends_with("hello world", "hello") { return assert(true, "str_ends_with absent"); } return assert(false, "str_ends_with absent"); }
fn test_str_ends_with_longer_suffix() -> TestResult { if !xiom.string.str_ends_with("hi", "hello") { return assert(true, "str_ends_with longer suffix"); } return assert(false, "str_ends_with longer suffix"); }

fn test_str_split_comma() -> TestResult { let parts = xiom.string.str_split("a,b,c", ","); if parts.len()==3 && parts.get(0).unwrap()=="a" && parts.get(1).unwrap()=="b" && parts.get(2).unwrap()=="c" { return assert(true, "str_split comma"); } return assert(false, "str_split comma"); }
fn test_str_split_space() -> TestResult { let parts = xiom.string.str_split("one two three", " "); if parts.len()==3 && parts.get(0)=="one" { return assert(true, "str_split space"); } return assert(false, "str_split space"); }
fn test_str_split_empty_delimiter() -> TestResult { let parts = xiom.string.str_split("abc", ""); if parts.len()==3 { return assert(true, "str_split empty delimiter"); } return assert(false, "str_split empty delimiter"); }
fn test_str_split_no_match() -> TestResult { let parts = xiom.string.str_split("hello", ","); if parts.len()==1 && parts.get(0).unwrap()=="hello" { return assert(true, "str_split no match"); } return assert(false, "str_split no match"); }

fn test_str_trim_whitespace() -> TestResult { if xiom.string.str_trim("  hello  ") == "hello" { return assert(true, "str_trim whitespace"); } return assert(false, "str_trim whitespace"); }
fn test_str_trim_no_whitespace() -> TestResult { if xiom.string.str_trim("hello") == "hello" { return assert(true, "str_trim no whitespace"); } return assert(false, "str_trim no whitespace"); }
fn test_str_trim_only_whitespace() -> TestResult { if xiom.string.str_trim("   ") == "" { return assert(true, "str_trim only whitespace"); } return assert(false, "str_trim only whitespace"); }

fn test_str_upper() -> TestResult { if xiom.string.str_upper("hello") == "HELLO" { return assert(true, "str_upper"); } return assert(false, "str_upper"); }
fn test_str_lower() -> TestResult { if xiom.string.str_lower("HELLO") == "hello" { return assert(true, "str_lower"); } return assert(false, "str_lower"); }
fn test_str_upper_already_upper() -> TestResult { if xiom.string.str_upper("ABC") == "ABC" { return assert(true, "str_upper already upper"); } return assert(false, "str_upper already upper"); }
fn test_str_lower_already_lower() -> TestResult { if xiom.string.str_lower("abc") == "abc" { return assert(true, "str_lower already lower"); } return assert(false, "str_lower already lower"); }

fn test_char_at_valid() -> TestResult { if xiom.string.char_at("hello", 0).unwrap() == 'h' { return assert(true, "char_at valid"); } return assert(false, "char_at valid"); }
fn test_char_at_out_of_bounds() -> TestResult { if xiom.string.char_at("hi", 5).is_none() { return assert(true, "char_at out of bounds"); } return assert(false, "char_at out of bounds"); }
fn test_char_at_negative() -> TestResult { if xiom.string.char_at("hi", -1).is_none() { return assert(true, "char_at negative"); } return assert(false, "char_at negative"); }

fn test_index_of_present() -> TestResult { if xiom.string.index_of("hello world", "world").unwrap() == 6 { return assert(true, "index_of present"); } return assert(false, "index_of present"); }
fn test_index_of_absent() -> TestResult { if xiom.string.index_of("hello world", "xyz").is_none() { return assert(true, "index_of absent"); } return assert(false, "index_of absent"); }
fn test_index_of_empty() -> TestResult { if xiom.string.index_of("hello", "").unwrap() == 0 { return assert(true, "index_of empty"); } return assert(false, "index_of empty"); }
fn test_index_of_at_start() -> TestResult { if xiom.string.index_of("abc", "a").unwrap() == 0 { return assert(true, "index_of at start"); } return assert(false, "index_of at start"); }
fn test_last_index_of_present() -> TestResult { if xiom.string.last_index_of("a,b,c,b", "b").unwrap() == 6 { return assert(true, "last_index_of present"); } return assert(false, "last_index_of present"); }

fn test_replace_single() -> TestResult { if xiom.string.replace("hello world", "world", "xiom") == "hello xiom" { return assert(true, "replace single"); } return assert(false, "replace single"); }
fn test_replace_multiple() -> TestResult { if xiom.string.replace("a,a,a", "a", "b") == "b,b,b" { return assert(true, "replace multiple"); } return assert(false, "replace multiple"); }
fn test_replace_no_match() -> TestResult { if xiom.string.replace("hello", "xyz", "abc") == "hello" { return assert(true, "replace no match"); } return assert(false, "replace no match"); }
fn test_replace_empty_from() -> TestResult { if xiom.string.replace("hello", "", "x") == "hello" { return assert(true, "replace empty from"); } return assert(false, "replace empty from"); }

fn test_lines() -> TestResult { let parts = xiom.string.lines("a\nb\nc"); if parts.len()==3 && parts.get(0).unwrap()=="a" { return assert(true, "lines"); } return assert(false, "lines"); }
fn test_lines_single() -> TestResult { let parts = xiom.string.lines("hello"); if parts.len()==1 && parts.get(0).unwrap()=="hello" { return assert(true, "lines single"); } return assert(false, "lines single"); }

fn test_words() -> TestResult { let parts = xiom.string.words("hello world xiom"); if parts.len()==3 && parts.get(0).unwrap()=="hello" { return assert(true, "words"); } return assert(false, "words"); }
fn test_words_single() -> TestResult { let parts = xiom.string.words("hello"); if parts.len()==1 && parts.get(0).unwrap()=="hello" { return assert(true, "words single"); } return assert(false, "words single"); }
fn test_words_extra_spaces() -> TestResult { let parts = xiom.string.words("  hello   world  "); if parts.len()==2 && parts.get(0).unwrap()=="hello" { return assert(true, "words extra spaces"); } return assert(false, "words extra spaces"); }

fn test_is_empty_true() -> TestResult { if xiom.string.is_empty("") { return assert(true, "is_empty true"); } return assert(false, "is_empty true"); }
fn test_is_empty_false() -> TestResult { if !xiom.string.is_empty("hello") { return assert(true, "is_empty false"); } return assert(false, "is_empty false"); }

fn test_char_count_empty() -> TestResult { if xiom.string.char_count("") == 0 { return assert(true, "char_count empty"); } return assert(false, "char_count empty"); }
fn test_char_count_nonempty() -> TestResult { if xiom.string.char_count("hello") == 5 { return assert(true, "char_count non-empty"); } return assert(false, "char_count non-empty"); }

fn test_byte_count_empty() -> TestResult { if xiom.string.byte_count("") == 0 { return assert(true, "byte_count empty"); } return assert(false, "byte_count empty"); }
fn test_byte_count_nonempty() -> TestResult { if xiom.string.byte_count("hello") == 5 { return assert(true, "byte_count non-empty"); } return assert(false, "byte_count non-empty"); }

fn main() -> Int {
  var tests: Vec[fn() -> TestResult] = [
    test_str_len_empty, test_str_len_nonempty,
    test_str_concat_two, test_str_concat_empty_first, test_str_concat_empty_second, test_str_concat_empty_both,
    test_str_slice_valid, test_str_slice_full, test_str_slice_middle, test_str_slice_empty_result,
    test_str_contains_present, test_str_contains_absent, test_str_contains_empty_substring,
    test_str_starts_with_present, test_str_starts_with_absent, test_str_starts_with_longer_prefix,
    test_str_ends_with_present, test_str_ends_with_absent, test_str_ends_with_longer_suffix,
    test_str_split_comma, test_str_split_space, test_str_split_empty_delimiter, test_str_split_no_match,
    test_str_trim_whitespace, test_str_trim_no_whitespace, test_str_trim_only_whitespace,
    test_str_upper, test_str_lower, test_str_upper_already_upper, test_str_lower_already_lower,
    test_char_at_valid, test_char_at_out_of_bounds, test_char_at_negative,
    test_index_of_present, test_index_of_absent, test_index_of_empty, test_index_of_at_start,
    test_last_index_of_present,
    test_replace_single, test_replace_multiple, test_replace_no_match, test_replace_empty_from,
    test_lines, test_lines_single,
    test_words, test_words_single, test_words_extra_spaces,
    test_is_empty_true, test_is_empty_false,
    test_char_count_empty, test_char_count_nonempty,
    test_byte_count_empty, test_byte_count_nonempty,
  ];
  return test.run_all(tests);
}
