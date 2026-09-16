// XIOM -- Ecosystem Networking Types Hardening Tests
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Self-contained networking type definitions and tests. Exercises
// enums, nested structs, Vec fields, method dispatch, and string building.

module tests.ecosystem.test_net

// ============================================================================
// Helper -- Int to String
// ============================================================================

fn int_to_str(n: Int) -> Str {
  if n == 0 { return "0"; }
  var neg = false;
  var val = n;
  if val < 0 { neg = true; val = -val; }
  var buf = "";
  while val > 0 {
    var digit = val % 10;
    val = val / 10;
    var ch = "";
    if digit == 0 { ch = "0"; }
    elif digit == 1 { ch = "1"; }
    elif digit == 2 { ch = "2"; }
    elif digit == 3 { ch = "3"; }
    elif digit == 4 { ch = "4"; }
    elif digit == 5 { ch = "5"; }
    elif digit == 6 { ch = "6"; }
    elif digit == 7 { ch = "7"; }
    elif digit == 8 { ch = "8"; }
    elif digit == 9 { ch = "9"; }
    buf = ch + buf;
  }
  if neg { buf = "-" + buf; }
  return buf;
}

// ============================================================================
// Type Definitions
// ============================================================================

pub enum IpVersion {
  V4,
  V6,
}

pub type IpAddr = {
  octets: Vec[Int];
  version: Int;
}

pub type SocketAddr = {
  ip: IpAddr;
  port: Int;
}

// ============================================================================
// IP Address Constructors
// ============================================================================

fn ipv4_new(a: Int, b: Int, c: Int, d: Int) -> IpAddr {
  var octets = Vec[Int].new();
  octets.push(a);
  octets.push(b);
  octets.push(c);
  octets.push(d);
  return IpAddr{ octets: octets, version: 4 };
}

fn ipv6_new(a: Int, b: Int, c: Int, d: Int, e: Int, f: Int, g: Int, h: Int) -> IpAddr {
  var octets = Vec[Int].new();
  octets.push(a);
  octets.push(b);
  octets.push(c);
  octets.push(d);
  octets.push(e);
  octets.push(f);
  octets.push(g);
  octets.push(h);
  return IpAddr{ octets: octets, version: 6 };
}

// ============================================================================
// Hex Helper
// ============================================================================

fn hex_digit(val: Int) -> Str {
  if val == 0 { return "0"; }
  elif val == 1 { return "1"; }
  elif val == 2 { return "2"; }
  elif val == 3 { return "3"; }
  elif val == 4 { return "4"; }
  elif val == 5 { return "5"; }
  elif val == 6 { return "6"; }
  elif val == 7 { return "7"; }
  elif val == 8 { return "8"; }
  elif val == 9 { return "9"; }
  elif val == 10 { return "a"; }
  elif val == 11 { return "b"; }
  elif val == 12 { return "c"; }
  elif val == 13 { return "d"; }
  elif val == 14 { return "e"; }
  elif val == 15 { return "f"; }
  return "0";
}

fn hex_byte(val: Int) -> Str {
  if val == 0 { return "00"; }
  var result = "";
  var v = val;
  while v > 0 {
    result = hex_digit(v % 16) + result;
    v = v / 16;
  }
  if val < 16 { result = "0" + result; }
  return result;
}

// ============================================================================
// IP Address Formatting -- Regular Functions
// ============================================================================

fn ipv4_to_str(ip: &IpAddr) -> Str {
  var result = "";
  var i = 0;
  while i < ip.octets.len() {
    if i > 0 { result = result + "."; }
    result = result + int_to_str(ip.octets[i]);
    i = i + 1;
  }
  return result;
}

fn ipv6_to_str(ip: &IpAddr) -> Str {
  var result = "";
  var i = 0;
  while i < 8 {
    if i > 0 { result = result + ":"; }
    var hi = ip.octets[i * 2];
    var lo = ip.octets[i * 2 + 1];
    var word = hi * 256 + lo;
    result = result + hex_byte(word);
    i = i + 1;
  }
  return result;
}

// ============================================================================
// IP Address Methods
// ============================================================================

fn IpAddr.is_v4() -> Bool {
  return this.version == 4;
}

fn IpAddr.is_v6() -> Bool {
  return this.version == 6;
}

fn IpAddr.octet(idx: Int) -> Option[Int] {
  if idx < 0 { return None; }
  if idx >= this.octets.len() { return None; }
  return Some(this.octets[idx]);
}

fn IpAddr.to_str() -> Str {
  if this.version == 4 { return ipv4_to_str(&this); }
  return ipv6_to_str(&this);
}

// ============================================================================
// Socket Address
// ============================================================================

fn socket_addr(ip: IpAddr, port: Int) -> SocketAddr {
  return SocketAddr{ ip: ip, port: port };
}

fn SocketAddr.to_str() -> Str {
  return IpAddr.to_str(&this.ip) + ":" + int_to_str(this.port);
}

// ============================================================================
// Test Functions -- IPv4
// ============================================================================

fn test_ipv4_new() -> Bool {
  let ip = ipv4_new(192, 168, 1, 1);
  return IpAddr.is_v4(&ip) && ip.octets.len() == 4;
}

fn test_ipv4_to_str() -> Bool {
  let ip = ipv4_new(192, 168, 1, 1);
  return ipv4_to_str(&ip) == "192.168.1.1";
}

fn test_ipv4_localhost() -> Bool {
  let ip = ipv4_new(127, 0, 0, 1);
  return ipv4_to_str(&ip) == "127.0.0.1";
}

fn test_ipv4_all_zeros() -> Bool {
  let ip = ipv4_new(0, 0, 0, 0);
  return ipv4_to_str(&ip) == "0.0.0.0";
}

fn test_ipv4_max_octets() -> Bool {
  let ip = ipv4_new(255, 255, 255, 255);
  return ipv4_to_str(&ip) == "255.255.255.255";
}

fn test_ipv4_octet_access() -> Bool {
  let ip = ipv4_new(10, 20, 30, 40);
  return IpAddr.octet(&ip, 0) == Some(10) &&
         IpAddr.octet(&ip, 1) == Some(20) &&
         IpAddr.octet(&ip, 2) == Some(30) &&
         IpAddr.octet(&ip, 3) == Some(40);
}

fn test_ipv4_octet_oob() -> Bool {
  let ip = ipv4_new(1, 2, 3, 4);
  return IpAddr.octet(&ip, -1).is_none() && IpAddr.octet(&ip, 4).is_none();
}

fn test_ipv4_method_to_str() -> Bool {
  let ip = ipv4_new(10, 0, 0, 1);
  return IpAddr.to_str(&ip) == "10.0.0.1";
}

fn test_ipv4_not_v6() -> Bool {
  let ip = ipv4_new(1, 1, 1, 1);
  return !IpAddr.is_v6(&ip);
}

// ============================================================================
// Test Functions -- IPv6
// ============================================================================

fn test_ipv6_new() -> Bool {
  let ip = ipv6_new(32, 1, 13, 184, 0, 0, 0, 1);
  return IpAddr.is_v6(&ip);
}

fn test_ipv6_localhost() -> Bool {
  let ip = ipv6_new(0, 0, 0, 0, 0, 0, 0, 1);
  return IpAddr.is_v6(&ip) && !IpAddr.is_v4(&ip);
}

fn test_ipv6_all_zeros() -> Bool {
  let ip = ipv6_new(0, 0, 0, 0, 0, 0, 0, 0);
  return ip.octets[0] == 0 && ip.octets[7] == 0;
}

fn test_ipv6_to_str_format() -> Bool {
  let ip = ipv6_new(0, 0, 0, 0, 0, 0, 0, 1);
  let s = ipv6_to_str(&ip);
  return s.len() > 0;
}

fn test_ipv6_method_to_str() -> Bool {
  let ip = ipv6_new(0, 0, 0, 0, 0, 0, 0, 1);
  let s = IpAddr.to_str(&ip);
  return s.len() > 0;
}

// ============================================================================
// Test Functions -- SocketAddr
// ============================================================================

fn test_socket_addr_new() -> Bool {
  let ip = ipv4_new(127, 0, 0, 1);
  let addr = socket_addr(ip, 8080);
  return addr.port == 8080;
}

fn test_socket_addr_to_str() -> Bool {
  let ip = ipv4_new(127, 0, 0, 1);
  let addr = socket_addr(ip, 8080);
  return SocketAddr.to_str(&addr) == "127.0.0.1:8080";
}

fn test_socket_addr_to_str_https() -> Bool {
  let ip = ipv4_new(192, 168, 1, 100);
  let addr = socket_addr(ip, 443);
  return SocketAddr.to_str(&addr) == "192.168.1.100:443";
}

fn test_socket_addr_ip_access() -> Bool {
  let ip = ipv4_new(10, 0, 0, 1);
  let addr = socket_addr(ip, 3000);
  return IpAddr.to_str(&addr.ip) == "10.0.0.1";
}

fn test_socket_addr_to_str_ipv6() -> Bool {
  let ip = ipv6_new(0, 0, 0, 0, 0, 0, 0, 1);
  let addr = socket_addr(ip, 80);
  let s = SocketAddr.to_str(&addr);
  return s.len() > 0;
}

// ============================================================================
// Test Functions -- IpVersion Enum
// ============================================================================

fn test_ip_version_enum() -> Bool {
  var v4 = IpVersion.V4;
  var v6 = IpVersion.V6;
  var ok_v4 = false;
  var ok_v6 = false;
  match v4 {
    V4 => { ok_v4 = true; }
    V6 => { ok_v4 = false; }
  }
  match v6 {
    V4 => { ok_v6 = false; }
    V6 => { ok_v6 = true; }
  }
  return ok_v4 && ok_v6;
}

// ============================================================================
// Test Functions -- Integration
// ============================================================================

fn test_multiple_addresses() -> Bool {
  var addresses = Vec[SocketAddr].new();
  let ip1 = ipv4_new(192, 168, 1, 1);
  let ip2 = ipv4_new(10, 0, 0, 1);
  let ip3 = ipv4_new(172, 16, 0, 1);
  addresses.push(socket_addr(ip1, 80));
  addresses.push(socket_addr(ip2, 443));
  addresses.push(socket_addr(ip3, 22));
  return addresses.len() == 3;
}

fn test_ipv4_roundtrip() -> Bool {
  let original = ipv4_new(192, 168, 1, 254);
  let s = ipv4_to_str(&original);
  var octets = Vec[Int].new();
  var i = 0;
  while i < 4 {
    if IpAddr.octet(&original, i).is_some() {
      octets.push(IpAddr.octet(&original, i).unwrap());
    }
    i = i + 1;
  }
  return octets[0] == 192 && octets[3] == 254;
}

// ============================================================================
// Main -- Run All Tests
// ============================================================================

fn main() -> Int {
  var passed = 0;
  var total = 0;

  total = total + 1; if test_ipv4_new() { passed = passed + 1; }
  total = total + 1; if test_ipv4_to_str() { passed = passed + 1; }
  total = total + 1; if test_ipv4_localhost() { passed = passed + 1; }
  total = total + 1; if test_ipv4_all_zeros() { passed = passed + 1; }
  total = total + 1; if test_ipv4_max_octets() { passed = passed + 1; }
  total = total + 1; if test_ipv4_octet_access() { passed = passed + 1; }
  total = total + 1; if test_ipv4_octet_oob() { passed = passed + 1; }
  total = total + 1; if test_ipv4_method_to_str() { passed = passed + 1; }
  total = total + 1; if test_ipv4_not_v6() { passed = passed + 1; }

  total = total + 1; if test_ipv6_new() { passed = passed + 1; }
  total = total + 1; if test_ipv6_localhost() { passed = passed + 1; }
  total = total + 1; if test_ipv6_all_zeros() { passed = passed + 1; }
  total = total + 1; if test_ipv6_to_str_format() { passed = passed + 1; }
  total = total + 1; if test_ipv6_method_to_str() { passed = passed + 1; }

  total = total + 1; if test_socket_addr_new() { passed = passed + 1; }
  total = total + 1; if test_socket_addr_to_str() { passed = passed + 1; }
  total = total + 1; if test_socket_addr_to_str_https() { passed = passed + 1; }
  total = total + 1; if test_socket_addr_ip_access() { passed = passed + 1; }
  total = total + 1; if test_socket_addr_to_str_ipv6() { passed = passed + 1; }

  total = total + 1; if test_ip_version_enum() { passed = passed + 1; }
  total = total + 1; if test_multiple_addresses() { passed = passed + 1; }
  total = total + 1; if test_ipv4_roundtrip() { passed = passed + 1; }

  if passed == total { return 0; }
  return 1;
}
