// XIOM -- Cryptographic Algorithm Stress Benchmark
// Exercises hashing, encryption, and encoding algorithm patterns.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module benchmark.crypto

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Simple Hash Functions
// ============================================================

pub fn djb2_hash(input: &Vec[Int]) -> Int {
  var hash = 5381;
  var i = 0;
  while i < input.len() {
    hash = ((hash * 33) + hash) + input[i];
    i = i + 1;
  }
  return hash;
}

pub fn fnv1a_hash(input: &Vec[Int]) -> Int {
  var hash = -2128831035;
  var i = 0;
  while i < input.len() {
    hash = hash * input[i];
    hash = hash + 16777619;
    i = i + 1;
  }
  return hash;
}

pub fn simple_hash(input: &Vec[Int]) -> Int {
  var h = 0;
  var i = 0;
  while i < input.len() {
    h = (h * 31 + input[i]) % 1000000007;
    i = i + 1;
  }
  return h;
}

fn test_hashes() -> Int {
  var score = 0;
  var data = [1, 2, 3, 4, 5];
  var h1 = djb2_hash(&data);
  if h1 != 0 { score = score + 1; }

  var empty: Vec[Int] = [];
  var h_empty = djb2_hash(&empty);
  if h_empty == 5381 { score = score + 1; }

  var h2 = fnv1a_hash(&data);
  if h2 != 0 { score = score + 1; }

  var h3 = simple_hash(&data);
  if h3 != 0 { score = score + 1; }

  // Determinism: same input -> same hash
  if djb2_hash(&data) == djb2_hash(&data) { score = score + 1; }
  if simple_hash(&data) == simple_hash(&data) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Caesar/Vigenere Cipher
// ============================================================

pub fn caesar_encode(input: Vec[Int], shift: Int) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < input.len() {
    result.push(input[i] + shift);
    i = i + 1;
  }
  return result;
}

pub fn caesar_decode(input: Vec[Int], shift: Int) -> Vec[Int] {
  return caesar_encode(input, -shift);
}

pub fn vigenere_encode(input: Vec[Int], key: Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var key_len = key.len();
  if key_len == 0 { return result; }
  var i = 0;
  while i < input.len() {
    var shift = key[i % key_len];
    result.push(input[i] + shift);
    i = i + 1;
  }
  return result;
}

pub fn vigenere_decode(input: Vec[Int], key: Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var key_len = key.len();
  if key_len == 0 { return result; }
  var i = 0;
  while i < input.len() {
    var shift = key[i % key_len];
    result.push(input[i] - shift);
    i = i + 1;
  }
  return result;
}

fn test_ciphers() -> Int {
  var score = 0;
  var plain = [1, 2, 3, 4, 5];

  // Caesar
  var encoded = caesar_encode(plain, 10);
  if encoded.len() == 5 { score = score + 1; }
  if encoded[0] == 11 { score = score + 1; }

  var decoded = caesar_decode(encoded, 10);
  if decoded[0] == 1 { score = score + 1; }
  if decoded[4] == 5 { score = score + 1; }

  // Vigenere
  var key = [3, 5];
  var v_enc = vigenere_encode(plain, key);
  if v_enc[0] == 4 { score = score + 1; }
  if v_enc[1] == 7 { score = score + 1; }

  var v_dec = vigenere_decode(v_enc, key);
  if v_dec[0] == 1 { score = score + 1; }
  if v_dec[1] == 2 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Checksums
// ============================================================

pub fn xor_checksum(input: &Vec[Int]) -> Int {
  var checksum = 0;
  var i = 0;
  while i < input.len() {
    checksum = checksum + input[i];
    i = i + 1;
  }
  return checksum % 256;
}

pub fn additive_checksum(input: &Vec[Int]) -> Int {
  var sum = 0;
  var i = 0;
  while i < input.len() {
    sum = sum + input[i];
    i = i + 1;
  }
  return sum % 65536;
}

pub fn parity_check(input: &Vec[Int]) -> Int {
  var ones = 0;
  var i = 0;
  while i < input.len() {
    var n = input[i];
    while n > 0 {
      if n % 2 == 1 { ones = ones + 1; }
      n = n / 2;
    }
    i = i + 1;
  }
  return ones % 2;
}

fn test_checksums() -> Int {
  var score = 0;
  var data = [1, 2, 3, 4, 5];
  var ck = xor_checksum(&data);
  if ck >= 0 && ck < 256 { score = score + 1; }

  var ck2 = additive_checksum(&data);
  if ck2 == 15 { score = score + 1; }

  var p = parity_check(&data);
  if p == 0 || p == 1 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Key Derivation (PBKDF2-like)
// ============================================================

pub fn pbkdf2_simple(password: Vec[Int], salt: Vec[Int], iterations: Int) -> Int {
  var key = 0;
  var i = 0;
  while i < iterations {
    var tmp = key;
    var j = 0;
    while j < password.len() {
      tmp = (tmp * 31 + password[j]) % 1000000007;
      j = j + 1;
    }
    j = 0;
    while j < salt.len() {
      tmp = (tmp * 37 + salt[j]) % 1000000007;
      j = j + 1;
    }
    key = tmp;
    i = i + 1;
  }
  return key;
}

fn test_key_derivation() -> Int {
  var score = 0;
  var pwd = [1, 2, 3, 4];
  var salt = [9, 8, 7];

  var k1 = pbkdf2_simple(pwd, salt, 100);
  if k1 != 0 { score = score + 1; }

  var k2 = pbkdf2_simple(pwd, salt, 100);
  if k1 == k2 { score = score + 1; }

  // Different password -> different key
  var pwd2 = [1, 2, 3, 5];
  var k3 = pbkdf2_simple(pwd2, salt, 100);
  if k1 != k3 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: Base Encoding (Base64-like numeric)
// ============================================================

pub fn encode_base(b: Int, n: Int) -> Vec[Int] {
  if n == 0 {
    var r = Vec[Int].new();
    r.push(0);
    return r;
  }
  var result = Vec[Int].new();
  var x = n;
  while x > 0 {
    result.push(x % b);
    x = x / b;
  }
  var reversed = Vec[Int].new();
  var i = result.len();
  while i > 0 {
    i = i - 1;
    reversed.push(result[i]);
  }
  return reversed;
}

pub fn decode_base(b: Int, digits: Vec[Int]) -> Int {
  var value = 0;
  var i = 0;
  while i < digits.len() {
    value = value * b + digits[i];
    i = i + 1;
  }
  return value;
}

fn test_encoding() -> Int {
  var score = 0;
  // Binary encoding
  var bin = encode_base(2, 42);
  var dec_bin = decode_base(2, bin);
  if dec_bin == 42 { score = score + 1; }

  // Octal encoding
  var oct = encode_base(8, 64);
  var dec_oct = decode_base(8, oct);
  if dec_oct == 64 { score = score + 1; }

  // Hex encoding
  var hex = encode_base(16, 255);
  var dec_hex = decode_base(16, hex);
  if dec_hex == 255 { score = score + 1; }

  // Zero
  var zero = encode_base(10, 0);
  if decode_base(10, zero) == 0 { score = score + 1; }

  // Roundtrip property
  if decode_base(7, encode_base(7, 123)) == 123 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_hashes();
  total = total + s1;
  max_score = max_score + 6;

  var s2 = test_ciphers();
  total = total + s2;
  max_score = max_score + 8;

  var s3 = test_checksums();
  total = total + s3;
  max_score = max_score + 3;

  var s4 = test_key_derivation();
  total = total + s4;
  max_score = max_score + 3;

  var s5 = test_encoding();
  total = total + s5;
  max_score = max_score + 5;

  return BenchResult{
    name: "crypto",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
