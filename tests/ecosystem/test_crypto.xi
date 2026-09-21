// XIOM -- Ecosystem Crypto Hardening Tests
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Self-contained cryptographic implementations: SHA-256, Base64,
// Hex encoding, and FNV-1a hash. Exercises bitwise operations,
// large constants, while loops, vector manipulation, and
// Result/Option types.

module tests.ecosystem.test_crypto
use xiom.collections;

// ============================================================================
// SHA-256: Initial hash values H[0..7]
// ============================================================================

fn sha256_h(i: Int) -> Int {
  if i == 0 { return 0x6a09e667; }
  elif i == 1 { return 0xbb67ae85; }
  elif i == 2 { return 0x3c6ef372; }
  elif i == 3 { return 0xa54ff53a; }
  elif i == 4 { return 0x510e527f; }
  elif i == 5 { return 0x9b05688c; }
  elif i == 6 { return 0x1f83d9ab; }
  elif i == 7 { return 0x5be0cd19; }
  return 0;
}

// ============================================================================
// SHA-256: Round constants K[0..63]
// ============================================================================

fn sha256_k(i: Int) -> Int {
  if i == 0 { return 0x428a2f98; }
  elif i == 1 { return 0x71374491; }
  elif i == 2 { return 0xb5c0fbcf; }
  elif i == 3 { return 0xe9b5dba5; }
  elif i == 4 { return 0x3956c25b; }
  elif i == 5 { return 0x59f111f1; }
  elif i == 6 { return 0x923f82a4; }
  elif i == 7 { return 0xab1c5ed5; }
  elif i == 8 { return 0xd807aa98; }
  elif i == 9 { return 0x12835b01; }
  elif i == 10 { return 0x243185be; }
  elif i == 11 { return 0x550c7dc3; }
  elif i == 12 { return 0x72be5d74; }
  elif i == 13 { return 0x80deb1fe; }
  elif i == 14 { return 0x9bdc06a7; }
  elif i == 15 { return 0xc19bf174; }
  elif i == 16 { return 0xe49b69c1; }
  elif i == 17 { return 0xefbe4786; }
  elif i == 18 { return 0x0fc19dc6; }
  elif i == 19 { return 0x240ca1cc; }
  elif i == 20 { return 0x2de92c6f; }
  elif i == 21 { return 0x4a7484aa; }
  elif i == 22 { return 0x5cb0a9dc; }
  elif i == 23 { return 0x76f988da; }
  elif i == 24 { return 0x983e5152; }
  elif i == 25 { return 0xa831c66d; }
  elif i == 26 { return 0xb00327c8; }
  elif i == 27 { return 0xbf597fc7; }
  elif i == 28 { return 0xc6e00bf3; }
  elif i == 29 { return 0xd5a79147; }
  elif i == 30 { return 0x06ca6351; }
  elif i == 31 { return 0x14292967; }
  elif i == 32 { return 0x27b70a85; }
  elif i == 33 { return 0x2e1b2138; }
  elif i == 34 { return 0x4d2c6dfc; }
  elif i == 35 { return 0x53380d13; }
  elif i == 36 { return 0x650a7354; }
  elif i == 37 { return 0x766a0abb; }
  elif i == 38 { return 0x81c2c92e; }
  elif i == 39 { return 0x92722c85; }
  elif i == 40 { return 0xa2bfe8a1; }
  elif i == 41 { return 0xa81a664b; }
  elif i == 42 { return 0xc24b8b70; }
  elif i == 43 { return 0xc76c51a3; }
  elif i == 44 { return 0xd192e819; }
  elif i == 45 { return 0xd6990624; }
  elif i == 46 { return 0xf40e3585; }
  elif i == 47 { return 0x106aa070; }
  elif i == 48 { return 0x19a4c116; }
  elif i == 49 { return 0x1e376c08; }
  elif i == 50 { return 0x2748774c; }
  elif i == 51 { return 0x34b0bcb5; }
  elif i == 52 { return 0x391c0cb3; }
  elif i == 53 { return 0x4ed8aa4a; }
  elif i == 54 { return 0x5b9cca4f; }
  elif i == 55 { return 0x682e6ff3; }
  elif i == 56 { return 0x748f82ee; }
  elif i == 57 { return 0x78a5636f; }
  elif i == 58 { return 0x84c87814; }
  elif i == 59 { return 0x8cc70208; }
  elif i == 60 { return 0x90befffa; }
  elif i == 61 { return 0xa4506ceb; }
  elif i == 62 { return 0xbef9a3f7; }
  elif i == 63 { return 0xc67178f2; }
  return 0;
}

// ============================================================================
// SHA-256: Bit manipulation helpers (32-bit unsigned simulation)
// ============================================================================

fn rotr(x: Int, n: Int) -> Int {
  return ((x >> n) | ((x << (32 - n)) & 0xFFFFFFFF)) & 0xFFFFFFFF;
}

fn ch(x: Int, y: Int, z: Int) -> Int {
  return ((x & y) ^ ((~x) & z)) & 0xFFFFFFFF;
}

fn maj(x: Int, y: Int, z: Int) -> Int {
  return ((x & y) ^ (x & z) ^ (y & z)) & 0xFFFFFFFF;
}

fn big_Sigma0(x: Int) -> Int {
  return (rotr(x, 2) ^ rotr(x, 13) ^ rotr(x, 22)) & 0xFFFFFFFF;
}

fn big_Sigma1(x: Int) -> Int {
  return (rotr(x, 6) ^ rotr(x, 11) ^ rotr(x, 25)) & 0xFFFFFFFF;
}

fn small_sigma0(x: Int) -> Int {
  return (rotr(x, 7) ^ rotr(x, 18) ^ (x >> 3)) & 0xFFFFFFFF;
}

fn small_sigma1(x: Int) -> Int {
  return (rotr(x, 17) ^ rotr(x, 19) ^ (x >> 10)) & 0xFFFFFFFF;
}

fn read_u32_be(data: &Vec[Int], offset: Int) -> Int {
  return ((data[offset] << 24) | (data[offset + 1] << 16) | (data[offset + 2] << 8) | data[offset + 3]) & 0xFFFFFFFF;
}

fn push_u32_be(v: &mut Vec[Int], val: Int) {
  v.push((val >> 24) & 0xFF);
  v.push((val >> 16) & 0xFF);
  v.push((val >> 8) & 0xFF);
  v.push(val & 0xFF);
}

fn push_u64_be(v: &mut Vec[Int], val: Int) {
  v.push((val >> 56) & 0xFF);
  v.push((val >> 48) & 0xFF);
  v.push((val >> 40) & 0xFF);
  v.push((val >> 32) & 0xFF);
  v.push((val >> 24) & 0xFF);
  v.push((val >> 16) & 0xFF);
  v.push((val >> 8) & 0xFF);
  v.push(val & 0xFF);
}

// ============================================================================
// SHA-256: Core compression
// ============================================================================

fn sha256_compress(h: &mut Vec[Int], chunk: &Vec[Int]) {
  var w = Vec[Int].new();
  var t = 0;
  while t < 16 {
    w.push(read_u32_be(chunk, t * 4));
    t = t + 1;
  }
  t = 16;
  while t < 64 {
    var s0 = small_sigma0(w[t - 15]);
    var s1 = small_sigma1(w[t - 2]);
    w.push((w[t - 16] + s0 + w[t - 7] + s1) & 0xFFFFFFFF);
    t = t + 1;
  }
  var a = h[0];
  var b = h[1];
  var c = h[2];
  var d = h[3];
  var e = h[4];
  var f = h[5];
  var g = h[6];
  var hh = h[7];
  t = 0;
  while t < 64 {
    var t1 = (hh + big_Sigma1(e) + ch(e, f, g) + sha256_k(t) + w[t]) & 0xFFFFFFFF;
    var t2 = (big_Sigma0(a) + maj(a, b, c)) & 0xFFFFFFFF;
    hh = g;
    g = f;
    f = e;
    e = (d + t1) & 0xFFFFFFFF;
    d = c;
    c = b;
    b = a;
    a = (t1 + t2) & 0xFFFFFFFF;
    t = t + 1;
  }
  h[0] = (h[0] + a) & 0xFFFFFFFF;
  h[1] = (h[1] + b) & 0xFFFFFFFF;
  h[2] = (h[2] + c) & 0xFFFFFFFF;
  h[3] = (h[3] + d) & 0xFFFFFFFF;
  h[4] = (h[4] + e) & 0xFFFFFFFF;
  h[5] = (h[5] + f) & 0xFFFFFFFF;
  h[6] = (h[6] + g) & 0xFFFFFFFF;
  h[7] = (h[7] + hh) & 0xFFFFFFFF;
}

// ============================================================================
// SHA-256: Main entry point -- returns 32-byte hash as Vec[Int]
// ============================================================================

fn sha256(data: &Vec[Int]) -> Vec[Int] {
  var padded = Vec[Int].new();
  var i = 0;
  while i < data.len() {
    padded.push(data[i]);
    i = i + 1;
  }
  padded.push(0x80);
  var bit_len = data.len() * 8;
  while ((padded.len() + 8) % 64) != 0 {
    padded.push(0);
  }
  push_u64_be(&mut padded, bit_len);
  var h = Vec[Int].new();
  i = 0;
  while i < 8 {
    h.push(sha256_h(i));
    i = i + 1;
  }
  var chunk = Vec[Int].new();
  var offset = 0;
  while offset < padded.len() {
    chunk.clear();
    i = 0;
    while i < 64 {
      chunk.push(padded[offset + i]);
      i = i + 1;
    }
    sha256_compress(&mut h, &chunk);
    offset = offset + 64;
  }
  var result = Vec[Int].new();
  i = 0;
  while i < 8 {
    push_u32_be(&mut result, h[i]);
    i = i + 1;
  }
  return result;
}

// ============================================================================
// Base64 Alphabet (as Vec[Str] for indexing)
// ============================================================================

fn base64_alphabet() -> Vec[Str] {
  return ["A","B","C","D","E","F","G","H","I","J","K","L","M","N","O","P","Q","R","S","T","U","V","W","X","Y","Z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","0","1","2","3","4","5","6","7","8","9","+","/"];
}

fn base64_index(ch: Str) -> Option[Int] {
  let alphabet = base64_alphabet();
  var i = 0;
  while i < 64 {
    if alphabet[i] == ch { return Some(i); }
    i = i + 1;
  }
  return None;
}

// ============================================================================
// Base64: Encode
// ============================================================================

fn base64_encode(data: &Vec[Int]) -> Str {
  let alphabet = base64_alphabet();
  var result = "";
  var i = 0;
  while i < data.len() {
    var b0 = data[i];
    if i + 2 < data.len() {
      var b1 = data[i + 1];
      var b2 = data[i + 2];
      var triple = (b0 << 16) | (b1 << 8) | b2;
      result = result + alphabet[(triple >> 18) & 0x3F];
      result = result + alphabet[(triple >> 12) & 0x3F];
      result = result + alphabet[(triple >> 6) & 0x3F];
      result = result + alphabet[triple & 0x3F];
      i = i + 3;
    } elif i + 1 < data.len() {
      var b1 = data[i + 1];
      var pair = (b0 << 16) | (b1 << 8);
      result = result + alphabet[(pair >> 18) & 0x3F];
      result = result + alphabet[(pair >> 12) & 0x3F];
      result = result + alphabet[(pair >> 6) & 0x3F];
      result = result + "=";
      i = i + 2;
    } else {
      var single = b0 << 16;
      result = result + alphabet[(single >> 18) & 0x3F];
      result = result + alphabet[(single >> 12) & 0x3F];
      result = result + "=";
      result = result + "=";
      i = i + 1;
    }
  }
  return result;
}

// ============================================================================
// Base64: Decode
// ============================================================================

fn base64_decode(input: Str) -> Result[Vec[Int], Str] {
  var result = Vec[Int].new();
  var i = 0;
  var len = input.len();
  if len % 4 != 0 {
    return Err("base64: input length must be a multiple of 4");
  }
  while i < len {
    var c0 = char_at_str(input, i);
    var c1 = char_at_str(input, i + 1);
    var c2 = char_at_str(input, i + 2);
    var c3 = char_at_str(input, i + 3);

    var v0 = base64_index(c0);
    var v1 = base64_index(c1);
    var pad2 = c2 == "=";
    var pad3 = c3 == "=";

    if v0.is_none() { return Err("base64: invalid character"); }
    if v1.is_none() { return Err("base64: invalid character"); }

    var triple = (v0.unwrap() << 18) | (v1.unwrap() << 12);
    if !pad2 {
      var v2 = base64_index(c2);
      if v2.is_none() { return Err("base64: invalid character"); }
      triple = triple | (v2.unwrap() << 6);
    }
    if !pad3 {
      var v3 = base64_index(c3);
      if v3.is_none() { return Err("base64: invalid character"); }
      triple = triple | v3.unwrap();
    }

    result.push((triple >> 16) & 0xFF);
    if !pad2 {
      result.push((triple >> 8) & 0xFF);
    }
    if !pad3 {
      result.push(triple & 0xFF);
    }
    i = i + 4;
  }
  return Ok(result);
}

fn char_at_str(s: Str, idx: Int) -> Str {
  return char_to_str(char_at(s, idx));
}

fn char_at(s: Str, idx: Int) -> Int {
  var i = 0;
  while i < s.len() {
    if i == idx {
      return s[i];
    }
    i = i + 1;
  }
  return 0;
}

fn str_slice(s: Str, start: Int, end: Int) -> Str {
  var result = "";
  var i = start;
  while i < end && i < s.len() {
    result = result + char_at_str(s, i);
    i = i + 1;
  }
  return result;
}

fn char_to_str(c: Int) -> Str {
  if c == 'A' { return "A"; }
  elif c == 'B' { return "B"; }
  elif c == 'C' { return "C"; }
  elif c == 'D' { return "D"; }
  elif c == 'E' { return "E"; }
  elif c == 'F' { return "F"; }
  elif c == 'G' { return "G"; }
  elif c == 'H' { return "H"; }
  elif c == 'I' { return "I"; }
  elif c == 'J' { return "J"; }
  elif c == 'K' { return "K"; }
  elif c == 'L' { return "L"; }
  elif c == 'M' { return "M"; }
  elif c == 'N' { return "N"; }
  elif c == 'O' { return "O"; }
  elif c == 'P' { return "P"; }
  elif c == 'Q' { return "Q"; }
  elif c == 'R' { return "R"; }
  elif c == 'S' { return "S"; }
  elif c == 'T' { return "T"; }
  elif c == 'U' { return "U"; }
  elif c == 'V' { return "V"; }
  elif c == 'W' { return "W"; }
  elif c == 'X' { return "X"; }
  elif c == 'Y' { return "Y"; }
  elif c == 'Z' { return "Z"; }
  elif c == 'a' { return "a"; }
  elif c == 'b' { return "b"; }
  elif c == 'c' { return "c"; }
  elif c == 'd' { return "d"; }
  elif c == 'e' { return "e"; }
  elif c == 'f' { return "f"; }
  elif c == 'g' { return "g"; }
  elif c == 'h' { return "h"; }
  elif c == 'i' { return "i"; }
  elif c == 'j' { return "j"; }
  elif c == 'k' { return "k"; }
  elif c == 'l' { return "l"; }
  elif c == 'm' { return "m"; }
  elif c == 'n' { return "n"; }
  elif c == 'o' { return "o"; }
  elif c == 'p' { return "p"; }
  elif c == 'q' { return "q"; }
  elif c == 'r' { return "r"; }
  elif c == 's' { return "s"; }
  elif c == 't' { return "t"; }
  elif c == 'u' { return "u"; }
  elif c == 'v' { return "v"; }
  elif c == 'w' { return "w"; }
  elif c == 'x' { return "x"; }
  elif c == 'y' { return "y"; }
  elif c == 'z' { return "z"; }
  elif c == '0' { return "0"; }
  elif c == '1' { return "1"; }
  elif c == '2' { return "2"; }
  elif c == '3' { return "3"; }
  elif c == '4' { return "4"; }
  elif c == '5' { return "5"; }
  elif c == '6' { return "6"; }
  elif c == '7' { return "7"; }
  elif c == '8' { return "8"; }
  elif c == '9' { return "9"; }
  elif c == ' ' { return " "; }
  elif c == '+' { return "+"; }
  elif c == '/' { return "/"; }
  elif c == '=' { return "="; }
  elif c == '\n' { return "\n"; }
  elif c == '\t' { return "\t"; }
  elif c == '\r' { return "\r"; }
  elif c == '-' { return "-"; }
  elif c == '_' { return "_"; }
  elif c == '.' { return "."; }
  elif c == ',' { return ","; }
  elif c == ':' { return ":"; }
  elif c == ';' { return ";"; }
  elif c == '"' { return "\""; }
  elif c == '\'' { return "'"; }
  return "";
}

// ============================================================================
// Hex Encoding
// ============================================================================

fn hex_chars() -> Vec[Str] {
  return ["0","1","2","3","4","5","6","7","8","9","a","b","c","d","e","f"];
}

fn hex_nibble_to_str(n: Int) -> Str {
  let chars = hex_chars();
  return chars[n & 0xF];
}

fn hex_encode(data: &Vec[Int]) -> Str {
  var result = "";
  var i = 0;
  while i < data.len() {
    result = result + hex_nibble_to_str((data[i] >> 4) & 0xF);
    result = result + hex_nibble_to_str(data[i] & 0xF);
    i = i + 1;
  }
  return result;
}

fn hex_char_to_val(ch: Str) -> Option[Int] {
  if ch == "0" { return Some(0); }
  elif ch == "1" { return Some(1); }
  elif ch == "2" { return Some(2); }
  elif ch == "3" { return Some(3); }
  elif ch == "4" { return Some(4); }
  elif ch == "5" { return Some(5); }
  elif ch == "6" { return Some(6); }
  elif ch == "7" { return Some(7); }
  elif ch == "8" { return Some(8); }
  elif ch == "9" { return Some(9); }
  elif ch == "a" || ch == "A" { return Some(10); }
  elif ch == "b" || ch == "B" { return Some(11); }
  elif ch == "c" || ch == "C" { return Some(12); }
  elif ch == "d" || ch == "D" { return Some(13); }
  elif ch == "e" || ch == "E" { return Some(14); }
  elif ch == "f" || ch == "F" { return Some(15); }
  return None;
}

fn hex_decode(input: Str) -> Result[Vec[Int], Str] {
  if input.len() % 2 != 0 {
    return Err("hex: input must have even length");
  }
  var result = Vec[Int].new();
  var i = 0;
  while i < input.len() {
    var hi_str = str_slice(input, i, i + 1);
    var lo_str = str_slice(input, i + 1, i + 2);
    var hi = hex_char_to_val(hi_str);
    var lo = hex_char_to_val(lo_str);
    if hi.is_none() || lo.is_none() {
      return Err("hex: invalid character");
    }
    result.push((hi.unwrap() << 4) | lo.unwrap());
    i = i + 2;
  }
  return Ok(result);
}

// ============================================================================
// FNV-1a Hash (64-bit)
// ============================================================================

fn fnv1a_hash(data: &Vec[Int]) -> Int {
  var hash = 14695981039346656037;
  var i = 0;
  while i < data.len() {
    hash = hash ^ data[i];
    hash = hash * 1099511628211;
    i = i + 1;
  }
  return hash;
}

// ============================================================================
// Tests
// ============================================================================

fn test_sha256_empty_known_vector() -> Bool {
  var empty = Vec[Int].new();
  let hash = sha256(&empty);
  let hex = hex_encode(&hash);
  return hex == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
}

fn test_sha256_length_32_bytes() -> Bool {
  var empty = Vec[Int].new();
  let hash = sha256(&empty);
  return hash.len() == 32;
}

fn test_sha256_deterministic() -> Bool {
  var data = Vec[Int].new();
  data.push(104);
  data.push(101);
  data.push(108);
  data.push(108);
  data.push(111);
  let h1 = sha256(&data);
  let h2 = sha256(&data);
  var i = 0;
  while i < 32 {
    if h1[i] != h2[i] { return false; }
    i = i + 1;
  }
  return true;
}

fn test_sha256_different_inputs_different_hash() -> Bool {
  var d1 = Vec[Int].new();
  d1.push(97);
  var d2 = Vec[Int].new();
  d2.push(98);
  let h1 = sha256(&d1);
  let h2 = sha256(&d2);
  var diff = false;
  var i = 0;
  while i < 32 {
    if h1[i] != h2[i] { diff = true; }
    i = i + 1;
  }
  return diff;
}

fn test_base64_encode_empty() -> Bool {
  var data = Vec[Int].new();
  return base64_encode(&data) == "";
}

fn test_base64_encode_man() -> Bool {
  var data = Vec[Int].new();
  data.push(77);
  data.push(97);
  data.push(110);
  return base64_encode(&data) == "TWFu";
}

fn test_base64_encode_single_byte() -> Bool {
  var data = Vec[Int].new();
  data.push(77);
  return base64_encode(&data) == "TQ==";
}

fn test_base64_encode_two_bytes() -> Bool {
  var data = Vec[Int].new();
  data.push(77);
  data.push(97);
  return base64_encode(&data) == "TWE=";
}

fn test_base64_encode_decode_roundtrip_hello() -> Bool {
  var data = Vec[Int].new();
  data.push(104);
  data.push(101);
  data.push(108);
  data.push(108);
  data.push(111);
  let encoded = base64_encode(&data);
  let decoded = base64_decode(encoded);
  if decoded.is_err() { return false; }
  let dec = decoded.unwrap();
  if dec.len() != data.len() { return false; }
  var i = 0;
  while i < data.len() {
    if dec[i] != data[i] { return false; }
    i = i + 1;
  }
  return true;
}

fn test_base64_decode_invalid_char() -> Bool {
  let result = base64_decode("!!!!");
  return result.is_err();
}

fn test_base64_decode_invalid_length() -> Bool {
  let result = base64_decode("abc");
  return result.is_err();
}

fn test_hex_encode_cafe() -> Bool {
  var data = Vec[Int].new();
  data.push(0xCA);
  data.push(0xFE);
  return hex_encode(&data) == "cafe";
}

fn test_hex_encode_empty() -> Bool {
  var data = Vec[Int].new();
  return hex_encode(&data) == "";
}

fn test_hex_encode_single_byte() -> Bool {
  var data = Vec[Int].new();
  data.push(0x0F);
  return hex_encode(&data) == "0f";
}

fn test_hex_encode_all_bytes() -> Bool {
  var data = Vec[Int].new();
  data.push(0x00);
  data.push(0xFF);
  return hex_encode(&data) == "00ff";
}

fn test_hex_decode_encode_roundtrip() -> Bool {
  var data = Vec[Int].new();
  data.push(0xDE);
  data.push(0xAD);
  data.push(0xBE);
  data.push(0xEF);
  let encoded = hex_encode(&data);
  let decoded = hex_decode(encoded);
  if decoded.is_err() { return false; }
  let dec = decoded.unwrap();
  if dec.len() != 4 { return false; }
  return dec[0] == 0xDE && dec[1] == 0xAD && dec[2] == 0xBE && dec[3] == 0xEF;
}

fn test_hex_decode_uppercase() -> Bool {
  let result = hex_decode("CAFE");
  if result.is_err() { return false; }
  let dec = result.unwrap();
  return dec.len() == 2 && dec[0] == 0xCA && dec[1] == 0xFE;
}

fn test_hex_decode_odd_length() -> Bool {
  let result = hex_decode("abc");
  return result.is_err();
}

fn test_hex_decode_invalid_char() -> Bool {
  let result = hex_decode("xy");
  return result.is_err();
}

fn test_hex_decode_empty() -> Bool {
  let result = hex_decode("");
  if result.is_err() { return false; }
  return result.unwrap().len() == 0;
}

fn test_fnv1a_empty() -> Bool {
  var data = Vec[Int].new();
  let hash = fnv1a_hash(&data);
  return hash == 14695981039346656037;
}

fn test_fnv1a_hello() -> Bool {
  var data = Vec[Int].new();
  data.push(104);
  data.push(101);
  data.push(108);
  data.push(108);
  data.push(111);
  let hash = fnv1a_hash(&data);
  return hash != 0;
}

fn test_fnv1a_deterministic() -> Bool {
  var data = Vec[Int].new();
  data.push(1);
  data.push(2);
  data.push(3);
  let h1 = fnv1a_hash(&data);
  let h2 = fnv1a_hash(&data);
  return h1 == h2;
}

fn test_fnv1a_different_input() -> Bool {
  var d1 = Vec[Int].new();
  d1.push(1);
  var d2 = Vec[Int].new();
  d2.push(2);
  return fnv1a_hash(&d1) != fnv1a_hash(&d2);
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Int {
  var passed = 0;
  var total = 0;

  total = total + 1;
  if test_sha256_length_32_bytes() { passed = passed + 1; }

  total = total + 1;
  if test_sha256_deterministic() { passed = passed + 1; }

  total = total + 1;
  if test_sha256_different_inputs_different_hash() { passed = passed + 1; }

  total = total + 1;
  if test_base64_encode_empty() { passed = passed + 1; }

  total = total + 1;
  if test_base64_encode_man() { passed = passed + 1; }

  total = total + 1;
  if test_base64_encode_single_byte() { passed = passed + 1; }

  total = total + 1;
  if test_base64_encode_two_bytes() { passed = passed + 1; }

  total = total + 1;
  if test_base64_encode_decode_roundtrip_hello() { passed = passed + 1; }

  total = total + 1;
  if test_base64_decode_invalid_char() { passed = passed + 1; }

  total = total + 1;
  if test_base64_decode_invalid_length() { passed = passed + 1; }

  total = total + 1;
  if test_hex_encode_cafe() { passed = passed + 1; }

  total = total + 1;
  if test_hex_encode_empty() { passed = passed + 1; }

  total = total + 1;
  if test_hex_encode_single_byte() { passed = passed + 1; }

  total = total + 1;
  if test_hex_encode_all_bytes() { passed = passed + 1; }

  total = total + 1;
  if test_hex_decode_encode_roundtrip() { passed = passed + 1; }

  total = total + 1;
  if test_hex_decode_uppercase() { passed = passed + 1; }

  total = total + 1;
  if test_hex_decode_odd_length() { passed = passed + 1; }

  total = total + 1;
  if test_hex_decode_invalid_char() { passed = passed + 1; }

  total = total + 1;
  if test_hex_decode_empty() { passed = passed + 1; }

  total = total + 1;
  if test_fnv1a_empty() { passed = passed + 1; }

  total = total + 1;
  if test_fnv1a_hello() { passed = passed + 1; }

  total = total + 1;
  if test_fnv1a_deterministic() { passed = passed + 1; }

  total = total + 1;
  if test_fnv1a_different_input() { passed = passed + 1; }

  if passed == total { return 0; }
  return 1;
}
