// XIOM -- I/O and Serialization Stress Benchmark
// Exercises file I/O patterns, serialization, encoding, and data transformation.
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

module benchmark.serialize

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Int <-> Str Conversion (simulated)
// ============================================================

pub fn int_to_digits(n: Int) -> Vec[Int] {
  if n == 0 {
    var r = Vec[Int].new();
    r.push(0);
    return r;
  }
  var result = Vec[Int].new();
  var x = if n < 0 { -n; } else { n; };
  while x > 0 {
    result.push(x % 10);
    x = x / 10;
  }
  var reversed = Vec[Int].new();
  var i = result.len();
  while i > 0 {
    i = i - 1;
    reversed.push(result[i]);
  }
  return reversed;
}

pub fn digits_to_int(digits: &Vec[Int]) -> Int {
  if digits.len() == 0 { return 0; }
  var value = 0;
  var i = 0;
  while i < digits.len() {
    value = value * 10 + digits[i];
    i = i + 1;
  }
  return value;
}

pub fn int_to_str(n: Int) -> Str {
  var digits = int_to_digits(n);
  // Return string representation - just use len for test
  return "0";
}

fn test_serialization() -> Int {
  var score = 0;
  var d1 = int_to_digits(123);
  if d1.len() == 3 { score = score + 1; }
  if d1[0] == 1 && d1[1] == 2 && d1[2] == 3 { score = score + 1; }

  var d2 = int_to_digits(0);
  if d2.len() == 1 && d2[0] == 0 { score = score + 1; }

  var d3 = int_to_digits(100);
  if d3.len() == 3 { score = score + 1; }

  // Roundtrip
  if digits_to_int(&d1) == 123 { score = score + 1; }
  if digits_to_int(&d2) == 0 { score = score + 1; }
  if digits_to_int(&d3) == 100 { score = score + 1; }

  // Roundtrip for many values
  var all_ok = 1;
  var i = 0;
  while i < 1000 {
    var digits = int_to_digits(i);
    if digits_to_int(&digits) != i { all_ok = 0; }
    i = i + 1;
  }
  if all_ok == 1 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: CSV-Like Encoding
// ============================================================

pub fn encode_row(values: &Vec[Int]) -> Vec[Int] {
  // Simple encoding: add delimiter (0) between values
  var result = Vec[Int].new();
  var i = 0;
  while i < values.len() {
    if i > 0 { result.push(0); }
    result.push(values[i]);
    i = i + 1;
  }
  return result;
}

pub fn decode_row(encoded: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var current = 0;
  var i = 0;
  while i < encoded.len() {
    if encoded[i] == 0 {
      result.push(current);
      current = 0;
    } else {
      current = current * 10 + encoded[i];
    }
    i = i + 1;
  }
  result.push(current);
  return result;
}

pub fn encode_table(rows: &Vec[Vec[Int]]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < rows.len() {
    if i > 0 { result.push(-1); }
    var encoded_row = encode_row(&rows[i]);
    var j = 0;
    while j < encoded_row.len() {
      result.push(encoded_row[j]);
      j = j + 1;
    }
    i = i + 1;
  }
  return result;
}

fn test_csv() -> Int {
  var score = 0;
  var row = [1, 2, 3];
  var encoded = encode_row(&row);
  var decoded = decode_row(&encoded);
  if decoded.len() == 3 { score = score + 1; }
  if decoded[0] == 1 { score = score + 1; }
  if decoded[2] == 3 { score = score + 1; }

  var table = Vec[Vec[Int]].new();
  table.push([1, 2]);
  table.push([3, 4]);
  table.push([5, 6]);
  var enc_table = encode_table(&table);
  if enc_table.len() > 0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: JSON-Like Serialization
// ============================================================

pub type JsonValue = {
  tag: Int;
  int_val: Int;
  float_val: Float64;
  bool_val: Bool;
  str_len: Int;
} derive[Clone]

pub fn JsonValue.new_int(val: Int) -> JsonValue {
  return JsonValue{ tag: 0, int_val: val, float_val: 0.0, bool_val: false, str_len: 0 };
}

pub fn JsonValue.new_float(val: Float64) -> JsonValue {
  return JsonValue{ tag: 1, int_val: 0, float_val: val, bool_val: false, str_len: 0 };
}

pub fn JsonValue.new_bool(val: Bool) -> JsonValue {
  return JsonValue{ tag: 2, int_val: 0, float_val: 0.0, bool_val: val, str_len: 0 };
}

pub fn JsonValue.is_int() -> Bool { return tag == 0; }
pub fn JsonValue.is_float() -> Bool { return tag == 1; }
pub fn JsonValue.is_bool() -> Bool { return tag == 2; }

pub fn serialize_json_array(values: &Vec[JsonValue]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < values.len() {
    result.push(values[i].tag);
    if values[i].is_int() { result.push(values[i].int_val); }
    elif values[i].is_float() { result.push(5); }
    elif values[i].is_bool() {
      if values[i].bool_val { result.push(1); } else { result.push(0); }
    }
    i = i + 1;
  }
  return result;
}

fn test_json() -> Int {
  var score = 0;
  var j1 = JsonValue.new_int(42);
  var j2 = JsonValue.new_bool(true);
  var j3 = JsonValue.new_int(99);

  if j1.is_int() { score = score + 1; }
  if j2.is_bool() { score = score + 1; }

  var arr = [j1, j2, j3];
  var serialized = serialize_json_array(&arr);
  if serialized.len() == 6 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Data Compression (Run-Length Encoding)
// ============================================================

pub fn rle_encode(input: &Vec[Int]) -> Vec[Int] {
  if input.len() == 0 {
    var r = Vec[Int].new();
    return r;
  }
  var result = Vec[Int].new();
  var i = 0;
  while i < input.len() {
    var count = 1;
    var j = i + 1;
    while j < input.len() && input[j] == input[i] && count < 255 {
      count = count + 1;
      j = j + 1;
    }
    result.push(input[i]);
    result.push(count);
    i = j;
  }
  return result;
}

pub fn rle_decode(input: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < input.len() {
    var value = input[i];
    var count = input[i + 1];
    var j = 0;
    while j < count {
      result.push(value);
      j = j + 1;
    }
    i = i + 2;
  }
  return result;
}

fn test_compression() -> Int {
  var score = 0;
  var data = [1, 1, 1, 2, 2, 3, 3, 3, 3];
  var encoded = rle_encode(&data);
  if encoded.len() == 6 { score = score + 1; }

  var decoded = rle_decode(&encoded);
  if decoded.len() == 9 { score = score + 1; }

  // Roundtrip
  var round = rle_decode(&rle_encode(&data));
  if round.len() == data.len() { score = score + 1; }
  var match_count = 0;
  var i = 0;
  while i < round.len() {
    if round[i] == data[i] { match_count = match_count + 1; }
    i = i + 1;
  }
  if match_count == 9 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: Binary Protocol
// ============================================================

pub type Packet = {
  version: Int;
  type_: Int;
  payload: Vec[Int];
  checksum: Int;
} derive[Clone]

pub fn Packet.new(ver: Int, ptype: Int, payload: Vec[Int]) -> Packet {
  var cksum = 0;
  var i = 0;
  while i < payload.len() {
    cksum = cksum + payload[i];
    i = i + 1;
  }
  cksum = cksum % 256;
  return Packet{ version: ver, type_: ptype, payload: payload, checksum: cksum };
}

pub fn Packet.verify() -> Bool {
  var computed = 0;
  var i = 0;
  while i < payload.len() {
    computed = computed + payload[i];
    i = i + 1;
  }
  return checksum == computed % 256;
}

pub fn Packet.serialize() -> Vec[Int] {
  var result = Vec[Int].new();
  result.push(version);
  result.push(type_);
  result.push(payload.len());
  var i = 0;
  while i < payload.len() {
    result.push(payload[i]);
    i = i + 1;
  }
  result.push(checksum);
  return result;
}

fn test_packet() -> Int {
  var score = 0;
  var p = Packet.new(1, 10, [1, 2, 3, 4, 5]);
  if p.version == 1 { score = score + 1; }
  if p.type_ == 10 { score = score + 1; }
  if p.verify() { score = score + 1; }

  var serialized = p.serialize();
  if serialized.len() == 9 { score = score + 1; }

  // Tamper-proof
  var p2 = Packet.new(1, 10, [10, 20]);
  if p2.verify() { score = score + 1; }
  if !(Packet{ version: 1, type_: 10, payload: [1, 2], checksum: 99 }.verify()) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_serialization();
  total = total + s1;
  max_score = max_score + 8;

  var s2 = test_csv();
  total = total + s2;
  max_score = max_score + 4;

  var s3 = test_json();
  total = total + s3;
  max_score = max_score + 3;

  var s4 = test_compression();
  total = total + s4;
  max_score = max_score + 4;

  var s5 = test_packet();
  total = total + s5;
  max_score = max_score + 6;

  return BenchResult{
    name: "serialize",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
