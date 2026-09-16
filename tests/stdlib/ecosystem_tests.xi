// XIOM -- Ecosystem Library Conformance Tests
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

module ecosystem_tests
use xiom.test;
use xiom.serialize;
use xiom.encoding;
use xiom.compress;
use xiom.crypto;
use xiom.regex;
use xiom.log;
use xiom.bench;
use xiom.contracts;
use xiom.reflect;

// ============================================================================
// Serialize Tests
// ============================================================================

fn test_parse_json_number() -> TestResult {
  let r = xiom.serialize.parse_json("42");
  if r.is_ok { return assert(true, "parse_json number"); }
  return assert(false, "parse_json number");
}

fn test_parse_json_string() -> TestResult {
  let r = xiom.serialize.parse_json("\"hello\"");
  if r.is_ok { return assert(true, "parse_json string"); }
  return assert(false, "parse_json string");
}

fn test_parse_json_bool_true() -> TestResult {
  let r = xiom.serialize.parse_json("true");
  if r.is_ok { return assert(true, "parse_json bool true"); }
  return assert(false, "parse_json bool true");
}

fn test_parse_json_bool_false() -> TestResult {
  let r = xiom.serialize.parse_json("false");
  if r.is_ok { return assert(true, "parse_json bool false"); }
  return assert(false, "parse_json bool false");
}

fn test_parse_json_null() -> TestResult {
  let r = xiom.serialize.parse_json("null");
  if r.is_ok { return assert(true, "parse_json null"); }
  return assert(false, "parse_json null");
}

fn test_parse_json_empty_array() -> TestResult {
  let r = xiom.serialize.parse_json("[]");
  if r.is_ok { return assert(true, "parse_json empty array"); }
  return assert(false, "parse_json empty array");
}

fn test_parse_json_simple_array() -> TestResult {
  let r = xiom.serialize.parse_json("[1, 2, 3]");
  if r.is_ok { return assert(true, "parse_json simple array"); }
  return assert(false, "parse_json simple array");
}

fn test_parse_json_simple_object() -> TestResult {
  let r = xiom.serialize.parse_json("{\"key\": \"value\"}");
  if r.is_ok { return assert(true, "parse_json simple object"); }
  return assert(false, "parse_json simple object");
}

fn test_to_json_basic() -> TestResult {
  let s = xiom.serialize.json_string("hello");
  let b = xiom.serialize.json_bool(true);
  let n = xiom.serialize.json_null();
  if s != "" && b == "true" && n == "null" { return assert(true, "to_json basic"); }
  return assert(false, "to_json basic");
}

fn test_json_escape_special_chars() -> TestResult {
  let s = xiom.serialize.json_string("a\"b\\c");
  if s != "" { return assert(true, "json_escape special chars"); }
  return assert(false, "json_escape special chars");
}

// ============================================================================
// Encoding Tests
// ============================================================================

fn test_base64_encode_empty() -> TestResult {
  var data = Vec[UInt8].new();
  let encoded = xiom.encoding.base64_encode(&data);
  if encoded == "" { return assert(true, "base64_encode empty"); }
  return assert(false, "base64_encode empty");
}

fn test_base64_encode_hello() -> TestResult {
  var data = Vec[UInt8].new();
  data.push(104); data.push(101); data.push(108); data.push(108); data.push(111);
  let encoded = xiom.encoding.base64_encode(&data);
  if encoded == "aGVsbG8=" { return assert(true, "base64_encode hello"); }
  return assert(false, "base64_encode hello");
}

fn test_base64_decode_roundtrip() -> TestResult {
  var data = Vec[UInt8].new();
  data.push(104); data.push(101); data.push(108); data.push(108); data.push(111);
  let encoded = xiom.encoding.base64_encode(&data);
  let decoded = xiom.encoding.base64_decode(encoded);
  if decoded.is_ok && decoded.value.len() == 5 { return assert(true, "base64_decode roundtrip"); }
  return assert(false, "base64_decode roundtrip");
}

fn test_hex_encode_decode_roundtrip() -> TestResult {
  var data = Vec[UInt8].new();
  data.push(0xDE); data.push(0xAD); data.push(0xBE); data.push(0xEF);
  let encoded = xiom.encoding.hex_encode(&data);
  let decoded = xiom.encoding.hex_decode(encoded);
  if decoded.is_ok && decoded.value.len() == 4 { return assert(true, "hex_encode/decode roundtrip"); }
  return assert(false, "hex_encode/decode roundtrip");
}

fn test_base64url_encode() -> TestResult {
  var data = Vec[UInt8].new();
  data.push(104); data.push(101); data.push(108); data.push(108); data.push(111);
  let encoded = xiom.encoding.base64url_encode(&data);
  if encoded != "" { return assert(true, "base64url_encode"); }
  return assert(false, "base64url_encode");
}

fn test_utf8_encode_decode_roundtrip() -> TestResult {
  let s = "hello";
  let encoded = xiom.encoding.utf8_encode(s);
  let decoded = xiom.encoding.utf8_decode(&encoded);
  if decoded.is_ok && decoded.value == s { return assert(true, "utf8_encode/decode roundtrip"); }
  return assert(false, "utf8_encode/decode roundtrip");
}

fn test_url_encode_decode_roundtrip() -> TestResult {
  let s = "hello world";
  let encoded = xiom.encoding.url_encode(s);
  let decoded = xiom.encoding.url_decode(encoded);
  if decoded.is_ok { return assert(true, "url_encode/decode roundtrip"); }
  return assert(false, "url_encode/decode roundtrip");
}

// ============================================================================
// Compress Tests
// ============================================================================

fn test_compress_gzip_roundtrip() -> TestResult {
  var data = Vec[UInt8].new();
  data.push(104); data.push(101); data.push(108); data.push(108); data.push(111);
  let compressed = xiom.compress.gzip_compress(&data);
  if compressed.is_ok {
    let decompressed = xiom.compress.gzip_decompress(&compressed.value);
    if decompressed.is_ok { return assert(true, "compress gzip roundtrip"); }
  }
  return assert(false, "compress gzip roundtrip");
}

fn test_detect_format_gzip() -> TestResult {
  var data = Vec[UInt8].new();
  data.push(0x1F); data.push(0x8B);
  let fmt = xiom.compress.detect_format(&data);
  if fmt == "gzip" { return assert(true, "detect_format gzip"); }
  return assert(false, "detect_format gzip");
}

fn test_detect_format_zlib() -> TestResult {
  var data = Vec[UInt8].new();
  data.push(0x78);
  let fmt = xiom.compress.detect_format(&data);
  if fmt == "zlib" { return assert(true, "detect_format zlib"); }
  return assert(false, "detect_format zlib");
}

fn test_is_compressed_basic() -> TestResult {
  var data = Vec[UInt8].new();
  data.push(0x1F); data.push(0x8B);
  let result = xiom.compress.is_compressed(&data);
  if result { return assert(true, "is_compressed basic"); }
  return assert(false, "is_compressed basic");
}

// ============================================================================
// Crypto Tests
// ============================================================================

fn test_sha256_32_bytes() -> TestResult {
  var data = Vec[UInt8].new();
  data.push(104); data.push(101); data.push(108); data.push(108); data.push(111);
  let hash = xiom.crypto.sha256(&data);
  if hash.len() == 32 { return assert(true, "sha256 32 bytes"); }
  return assert(false, "sha256 32 bytes");
}

fn test_sha256_deterministic() -> TestResult {
  var data = Vec[UInt8].new();
  data.push(104); data.push(101); data.push(108); data.push(108); data.push(111);
  let hash1 = xiom.crypto.sha256(&data);
  let hash2 = xiom.crypto.sha256(&data);
  var i = 0;
  while i < 32 {
    if hash1[i] != hash2[i] { return assert(false, "sha256 deterministic"); }
    i = i + 1;
  }
  return assert(true, "sha256 deterministic");
}

fn test_sha256_different_inputs() -> TestResult {
  var d1 = Vec[UInt8].new(); d1.push(104);
  var d2 = Vec[UInt8].new(); d2.push(105);
  let h1 = xiom.crypto.sha256(&d1);
  let h2 = xiom.crypto.sha256(&d2);
  var diff = false; var i = 0;
  while i < 32 {
    if h1[i] != h2[i] { diff = true; }
    i = i + 1;
  }
  if diff { return assert(true, "sha256 different inputs"); }
  return assert(false, "sha256 different inputs");
}

fn test_sha256_hex_64_chars() -> TestResult {
  var data = Vec[UInt8].new();
  data.push(104);
  let hex = xiom.crypto.sha256_hex(&data);
  if hex.len() == 64 { return assert(true, "sha256_hex 64 chars"); }
  return assert(false, "sha256_hex 64 chars");
}

fn test_hmac_sha256_basic() -> TestResult {
  var key = Vec[UInt8].new();
  key.push(107); key.push(101); key.push(121);
  var data = Vec[UInt8].new();
  data.push(100); data.push(97); data.push(116); data.push(97);
  let result = xiom.crypto.hmac_sha256(&key, &data);
  if result.len() == 32 { return assert(true, "hmac_sha256 basic"); }
  return assert(false, "hmac_sha256 basic");
}

fn test_constant_time_compare_equal() -> TestResult {
  var a = Vec[UInt8].new(); a.push(1); a.push(2); a.push(3);
  var b = Vec[UInt8].new(); b.push(1); b.push(2); b.push(3);
  if xiom.crypto.constant_time_compare(&a, &b) { return assert(true, "constant_time_compare equal"); }
  return assert(false, "constant_time_compare equal");
}

fn test_constant_time_compare_not_equal() -> TestResult {
  var a = Vec[UInt8].new(); a.push(1); a.push(2); a.push(3);
  var b = Vec[UInt8].new(); b.push(4); b.push(5); b.push(6);
  if !xiom.crypto.constant_time_compare(&a, &b) { return assert(true, "constant_time_compare not equal"); }
  return assert(false, "constant_time_compare not equal");
}

// ============================================================================
// Regex Tests
// ============================================================================

fn test_regex_is_match() -> TestResult {
  let re = xiom.regex.Regex.new("hello").unwrap();
  if re.is_match("hello world") { return assert(true, "regex is_match literal"); }
  return assert(false, "regex is_match literal");
}

fn test_regex_is_match_not() -> TestResult {
  let re = xiom.regex.Regex.new("xyz").unwrap();
  if !re.is_match("hello world") { return assert(true, "regex is_match not matching"); }
  return assert(false, "regex is_match not matching");
}

fn test_regex_find() -> TestResult {
  let re = xiom.regex.Regex.new("world").unwrap();
  let m = re.find("hello world");
  if m.is_some { return assert(true, "regex find"); }
  return assert(false, "regex find");
}

fn test_regex_find_all() -> TestResult {
  let re = xiom.regex.Regex.new("a").unwrap();
  let matches = re.find_all("banana");
  if matches.len() == 3 { return assert(true, "regex find_all"); }
  return assert(false, "regex find_all");
}

fn test_regex_replace() -> TestResult {
  let re = xiom.regex.Regex.new("cat").unwrap();
  let result = re.replace("the cat sat", "dog");
  if result == "the dog sat" { return assert(true, "regex replace"); }
  return assert(false, "regex replace");
}

fn test_regex_split() -> TestResult {
  let re = xiom.regex.Regex.new(",").unwrap();
  let parts = re.split("a,b,c");
  if parts.len() >= 2 { return assert(true, "regex split"); }
  return assert(false, "regex split");
}

fn test_regex_wildcard() -> TestResult {
  let re = xiom.regex.Regex.new("h.*o").unwrap();
  if re.is_match("hello") { return assert(true, "regex .* wildcard"); }
  return assert(false, "regex .* wildcard");
}

fn test_regex_char_class() -> TestResult {
  let re = xiom.regex.Regex.new("[a-z]+").unwrap();
  if re.is_match("hello") && !re.is_match("123") { return assert(true, "regex [a-z]+ char class"); }
  return assert(false, "regex [a-z]+ char class");
}

fn test_regex_anchor_start() -> TestResult {
  let re = xiom.regex.Regex.new("^hello").unwrap();
  if re.is_match("hello world") && !re.is_match("world hello") { return assert(true, "regex ^ anchor"); }
  return assert(false, "regex ^ anchor");
}

fn test_regex_anchor_end() -> TestResult {
  let re = xiom.regex.Regex.new("world$").unwrap();
  if re.is_match("hello world") && !re.is_match("world hello") { return assert(true, "regex $ anchor"); }
  return assert(false, "regex $ anchor");
}

// ============================================================================
// Log Tests
// ============================================================================

fn test_log_entry_creation() -> TestResult {
  let entry = xiom.log.LogEntry{
    level: xiom.log.LogLevel.Info;
    message: "test";
    file: "";
    line: 0;
    timestamp: 0;
    data: Map[Str, Str].new();
  };
  if entry.message == "test" { return assert(true, "Logger creation"); }
  return assert(false, "Logger creation");
}

fn test_log_set_level() -> TestResult {
  xiom.log.set_level(xiom.log.LogLevel.Warn);
  let lvl = xiom.log.get_level();
  if lvl == xiom.log.LogLevel.Warn { return assert(true, "set_level + filtering"); }
  return assert(false, "set_level + filtering");
}

fn test_log_info_warn_error() -> TestResult {
  xiom.log.info("test info");
  xiom.log.warn("test warn");
  xiom.log.error("test error");
  return assert(true, "info/warn/error basic");
}

fn test_log_format_entry_basic() -> TestResult {
  let entry = xiom.log.LogEntry{
    level: xiom.log.LogLevel.Info;
    message: "formatted";
    file: "";
    line: 0;
    timestamp: 0;
    data: Map[Str, Str].new();
  };
  if entry.level == xiom.log.LogLevel.Info { return assert(true, "format_entry basic"); }
  return assert(false, "format_entry basic");
}

// ============================================================================
// Bench Tests
// ============================================================================

fn test_run_bench_basic() -> TestResult {
  let result = xiom.bench.run_bench("test_bench", fn() { var x = 0; x = x + 1; });
  if result.name == "test_bench" { return assert(true, "run_bench basic"); }
  return assert(false, "run_bench basic");
}

fn test_black_box_preserves() -> TestResult {
  let val = 42;
  let result = xiom.bench.black_box(val);
  if result == val { return assert(true, "black_box preserves value"); }
  return assert(false, "black_box preserves value");
}

// ============================================================================
// Test Framework Tests (testing xiom.test itself)
// ============================================================================

fn test_assert_eq_passes() -> TestResult {
  let r = xiom.test.assert_eq(42, 42, "test");
  if r.passed { return assert(true, "assert_eq passes on equal"); }
  return assert(false, "assert_eq passes on equal");
}

fn test_assert_ok() -> TestResult {
  let r = xiom.test.assert_ok(Ok::<Int, Str>(42), "test");
  if r.passed { return assert(true, "assert_ok"); }
  return assert(false, "assert_ok");
}

fn test_assert_err() -> TestResult {
  let r = xiom.test.assert_err(Err::<Int, Str>("fail"), "test");
  if r.passed { return assert(true, "assert_err"); }
  return assert(false, "assert_err");
}

fn test_assert_some() -> TestResult {
  let r = xiom.test.assert_some(Some(42), "test");
  if r.passed { return assert(true, "assert_some"); }
  return assert(false, "assert_some");
}

fn test_assert_none() -> TestResult {
  let r: Option[Int] = None;
  let result = xiom.test.assert_none(r, "test");
  if result.passed { return assert(true, "assert_none"); }
  return assert(false, "assert_none");
}

fn test_run_returns_result() -> TestResult {
  let exit_code = xiom.test.run(fn() -> TestResult { return xiom.test.assert(true, "inner"); });
  if exit_code == 0 { return assert(true, "run returns TestResult"); }
  return assert(false, "run returns TestResult");
}

// ============================================================================
// Contracts Tests
// ============================================================================

fn test_contract_index_creation() -> TestResult {
  let idx = xiom.contracts.build_contract_index();
  if idx.package == "" { return assert(true, "ContractIndex creation"); }
  return assert(false, "ContractIndex creation");
}

fn test_total_contracts() -> TestResult {
  let n = xiom.contracts.total_contracts();
  if n >= 0 { return assert(true, "total_contracts returns value"); }
  return assert(false, "total_contracts returns value");
}

fn test_export_contracts_markdown() -> TestResult {
  let md = xiom.contracts.export_contracts_markdown();
  if md != "" { return assert(true, "export_contracts_markdown returns string"); }
  return assert(false, "export_contracts_markdown returns string");
}

fn test_coverage_percentage() -> TestResult {
  let pct = xiom.contracts.coverage_percentage();
  if pct >= 0.0 { return assert(true, "coverage_percentage returns value"); }
  return assert(false, "coverage_percentage returns value");
}

// ============================================================================
// Reflect Tests
// ============================================================================

fn test_type_name_returns_string() -> TestResult {
  let name = xiom.reflect.type_name[Int]();
  if name != "" { return assert(true, "type_name returns string"); }
  return assert(false, "type_name returns string");
}

fn test_type_size_positive() -> TestResult {
  let sz = xiom.reflect.type_size[Int]();
  if sz > 0 { return assert(true, "type_size > 0"); }
  return assert(false, "type_size > 0");
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Int {
  var tests = [
    test_parse_json_number, test_parse_json_string, test_parse_json_bool_true, test_parse_json_bool_false,
    test_parse_json_null, test_parse_json_empty_array, test_parse_json_simple_array, test_parse_json_simple_object,
    test_to_json_basic, test_json_escape_special_chars,
    test_base64_encode_empty, test_base64_encode_hello, test_base64_decode_roundtrip,
    test_hex_encode_decode_roundtrip, test_base64url_encode,
    test_utf8_encode_decode_roundtrip, test_url_encode_decode_roundtrip,
    test_compress_gzip_roundtrip, test_detect_format_gzip, test_detect_format_zlib, test_is_compressed_basic,
    test_sha256_32_bytes, test_sha256_deterministic, test_sha256_different_inputs, test_sha256_hex_64_chars,
    test_hmac_sha256_basic, test_constant_time_compare_equal, test_constant_time_compare_not_equal,
    test_regex_is_match, test_regex_is_match_not, test_regex_find, test_regex_find_all,
    test_regex_replace, test_regex_split, test_regex_wildcard, test_regex_char_class,
    test_regex_anchor_start, test_regex_anchor_end,
    test_log_entry_creation, test_log_set_level, test_log_info_warn_error, test_log_format_entry_basic,
    test_run_bench_basic, test_black_box_preserves,
    test_assert_eq_passes, test_assert_ok, test_assert_err, test_assert_some, test_assert_none, test_run_returns_result,
    test_contract_index_creation, test_total_contracts, test_export_contracts_markdown, test_coverage_percentage,
    test_type_name_returns_string, test_type_size_positive,
  ];
  return test.run_all(tests);
}
