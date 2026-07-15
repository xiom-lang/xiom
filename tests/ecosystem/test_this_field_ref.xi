// XIOM — Regression Test: Nested this-based method dispatch via &this.field
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// Verifies that when a this-based method passes &this.field to another
// this-based method (e.g. SocketAddr.to_str calling IpAddr.to_str(&this.ip)),
// the GEP pointer is preserved instead of loading the field by value.
// Regression for ACCESS_VIOLATION (0xC0000005) in NET ecosystem tests.

module tests.ecosystem.test_this_field_ref

fn int_to_str(n: Int) -> Str {
  if n == 0 { return "0"; }
  var neg = false; var val = n;
  if val < 0 { neg = true; val = -val; }
  var buf = "";
  while val > 0 {
    var digit = val % 10; val = val / 10;
    var ch = "";
    if digit == 0 { ch = "0"; } elif digit == 1 { ch = "1"; }
    elif digit == 2 { ch = "2"; } elif digit == 3 { ch = "3"; }
    elif digit == 4 { ch = "4"; } elif digit == 5 { ch = "5"; }
    elif digit == 6 { ch = "6"; } elif digit == 7 { ch = "7"; }
    elif digit == 8 { ch = "8"; } elif digit == 9 { ch = "9"; }
    buf = ch + buf;
  }
  if neg { buf = "-" + buf; }
  return buf;
}

pub type IpAddr = { octets: Vec[Int]; version: Int; }
pub type SocketAddr = { ip: IpAddr; port: Int; }

fn ipv4_new(a: Int, b: Int, c: Int, d: Int) -> IpAddr {
  var octets = Vec[Int].new();
  octets.push(a); octets.push(b); octets.push(c); octets.push(d);
  return IpAddr{ octets: octets, version: 4 };
}

fn ipv4_to_str(ip: &IpAddr) -> Str {
  var result = ""; var i = 0;
  while i < ip.octets.len() {
    if i > 0 { result = result + "."; }
    result = result + int_to_str(ip.octets[i]);
    i = i + 1;
  }
  return result;
}

fn IpAddr.is_v4() -> Bool { return this.version == 4; }
fn IpAddr.to_str() -> Str { if this.version == 4 { return ipv4_to_str(&this); } return ""; }
fn socket_addr(ip: IpAddr, port: Int) -> SocketAddr { return SocketAddr{ ip: ip, port: port }; }
fn SocketAddr.to_str() -> Str { return IpAddr.to_str(&this.ip) + ":" + int_to_str(this.port); }

fn main() -> Int {
  var passed = 0; var total = 0;

  // Test 1: int_to_str correctness
  total = total + 1;
  if int_to_str(123) == "123" { passed = passed + 1; }

  // Test 2: int_to_str zero
  total = total + 1;
  if int_to_str(0) == "0" { passed = passed + 1; }

  // Test 3: IpAddr.is_v4 (this-based, top-level field)
  total = total + 1;
  let ip = ipv4_new(192, 168, 1, 1);
  if IpAddr.is_v4(&ip) { passed = passed + 1; }

  // Test 4: IpAddr.to_str via &this (this-based, passes self pointer)
  total = total + 1;
  let ip2 = ipv4_new(10, 0, 0, 1);
  if IpAddr.to_str(&ip2) == "10.0.0.1" { passed = passed + 1; }

  // Test 5: SocketAddr.to_str — calls IpAddr.to_str(&this.ip)
  // This is the key regression: &this.ip must return the GEP pointer
  // to the ip field, not the loaded IpAddr value.
  total = total + 1;
  let ip3 = ipv4_new(127, 0, 0, 1);
  let addr = socket_addr(ip3, 8080);
  if SocketAddr.to_str(&addr) == "127.0.0.1:8080" { passed = passed + 1; }

  // Test 6: Multiple calls to SocketAddr.to_str (different addresses)
  total = total + 1;
  let ip4 = ipv4_new(192, 168, 1, 100);
  let addr2 = socket_addr(ip4, 443);
  if SocketAddr.to_str(&addr2) == "192.168.1.100:443" { passed = passed + 1; }

  // Test 7: IpAddr.to_str on nested field from main (not within method)
  total = total + 1;
  let ip5 = ipv4_new(10, 0, 0, 1);
  let addr3 = socket_addr(ip5, 3000);
  if IpAddr.to_str(&addr3.ip) == "10.0.0.1" { passed = passed + 1; }

  if passed == total { return 0; }
  return 1;
}
