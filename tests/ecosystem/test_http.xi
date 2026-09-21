// XIOM -- Ecosystem HTTP Type Hardening Tests
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Self-contained HTTP type definitions and tests. Exercises enums,
// nested structs, Vec of structs, and method dispatch patterns.

module tests.ecosystem.test_http
use xiom.collections;

pub enum HttpMethod {
  GET,
  POST,
  PUT,
  DELETE,
  PATCH,
  HEAD,
  OPTIONS,
}

pub type HttpHeader = {
  name: Str;
  value: Str;
}

pub type HttpHeaders = {
  entries: Vec[HttpHeader];
}

pub type HttpRequest = {
  method: HttpMethod;
  path: Str;
  headers: HttpHeaders;
  body: Str;
}

pub type HttpResponse = {
  status: Int;
  headers: HttpHeaders;
  body: Str;
}

fn HttpHeaders.new() -> HttpHeaders {
  var entries = Vec[HttpHeader].new();
  return HttpHeaders{ entries: entries };
}

fn HttpHeaders.add(h: &mut HttpHeaders, name: Str, value: Str) {
  var header = HttpHeader{ name: name, value: value };
  h.entries.push(header);
}

fn HttpHeaders.get(h: &HttpHeaders, name: Str) -> Option[Str] {
  var i = 0;
  while i < h.entries.len() {
    if h.entries[i].name == name {
      return Some(h.entries[i].value);
    }
    i = i + 1;
  }
  return None;
}

fn HttpHeaders.has(h: &HttpHeaders, name: Str) -> Bool {
  var i = 0;
  while i < h.entries.len() {
    if h.entries[i].name == name {
      return true;
    }
    i = i + 1;
  }
  return false;
}

fn HttpHeaders.count(h: &HttpHeaders) -> Int {
  return h.entries.len();
}

fn HttpHeaders.remove(h: &mut HttpHeaders, name: Str) -> Bool {
  var i = 0;
  while i < h.entries.len() {
    if h.entries[i].name == name {
      h.entries.remove(i);
      return true;
    }
    i = i + 1;
  }
  return false;
}

fn HttpRequest.new(method: HttpMethod, path: Str) -> HttpRequest {
  var headers = HttpHeaders.new();
  return HttpRequest{ method: method, path: path, headers: headers, body: "" };
}

fn HttpRequest.set_header(req: &mut HttpRequest, name: Str, value: Str) {
  HttpHeaders.add(&mut req.headers, name, value);
}

fn HttpRequest.set_body(req: &mut HttpRequest, body: Str) {
  req.body = body;
}

fn HttpRequest.get_header(req: &HttpRequest, name: Str) -> Option[Str] {
  return HttpHeaders.get(&req.headers, name);
}

fn HttpRequest.has_header(req: &HttpRequest, name: Str) -> Bool {
  return HttpHeaders.has(&req.headers, name);
}

fn HttpResponse.new(status: Int) -> HttpResponse {
  var headers = HttpHeaders.new();
  return HttpResponse{ status: status, headers: headers, body: "" };
}

fn HttpResponse.set_header(res: &mut HttpResponse, name: Str, value: Str) {
  HttpHeaders.add(&mut res.headers, name, value);
}

fn HttpResponse.set_body(res: &mut HttpResponse, body: Str) {
  res.body = body;
}

fn HttpResponse.get_header(res: &HttpResponse, name: Str) -> Option[Str] {
  return HttpHeaders.get(&res.headers, name);
}

fn HttpResponse.has_header(res: &HttpResponse, name: Str) -> Bool {
  return HttpHeaders.has(&res.headers, name);
}

fn HttpMethod.to_str(method: HttpMethod) -> Str {
  match method {
    GET => { return "GET"; }
    POST => { return "POST"; }
    PUT => { return "PUT"; }
    DELETE => { return "DELETE"; }
    PATCH => { return "PATCH"; }
    HEAD => { return "HEAD"; }
    OPTIONS => { return "OPTIONS"; }
  }
}

fn HttpMethod.from_str(s: Str) -> Option[HttpMethod] {
  if s == "GET" { return Some(HttpMethod.GET); }
  elif s == "POST" { return Some(HttpMethod.POST); }
  elif s == "PUT" { return Some(HttpMethod.PUT); }
  elif s == "DELETE" { return Some(HttpMethod.DELETE); }
  elif s == "PATCH" { return Some(HttpMethod.PATCH); }
  elif s == "HEAD" { return Some(HttpMethod.HEAD); }
  elif s == "OPTIONS" { return Some(HttpMethod.OPTIONS); }
  return None;
}

// ============================================================================
// Tests
// ============================================================================

fn test_headers_new_empty() -> Bool {
  let h = HttpHeaders.new();
  return HttpHeaders.count(&h) == 0;
}

fn test_headers_add_and_get() -> Bool {
  var h = HttpHeaders.new();
  HttpHeaders.add(&mut h, "Content-Type", "application/json");
  let val = HttpHeaders.get(&h, "Content-Type");
  if val.is_some() { return val.unwrap() == "application/json"; }
  return false;
}

fn test_headers_add_multiple() -> Bool {
  var h = HttpHeaders.new();
  HttpHeaders.add(&mut h, "A", "1");
  HttpHeaders.add(&mut h, "B", "2");
  HttpHeaders.add(&mut h, "C", "3");
  return HttpHeaders.count(&h) == 3;
}

fn test_headers_get_missing() -> Bool {
  var h = HttpHeaders.new();
  HttpHeaders.add(&mut h, "X-Custom", "value");
  let val = HttpHeaders.get(&h, "NonExistent");
  return val.is_none();
}

fn test_headers_has() -> Bool {
  var h = HttpHeaders.new();
  HttpHeaders.add(&mut h, "Authorization", "Bearer token");
  let has_auth = HttpHeaders.has(&h, "Authorization");
  let has_other = HttpHeaders.has(&h, "Other");
  return has_auth && !has_other;
}

fn test_headers_remove() -> Bool {
  var h = HttpHeaders.new();
  HttpHeaders.add(&mut h, "X-Temp", "42");
  HttpHeaders.add(&mut h, "X-Keep", "7");
  let removed = HttpHeaders.remove(&mut h, "X-Temp");
  return removed && HttpHeaders.count(&h) == 1 && !HttpHeaders.has(&h, "X-Temp");
}

fn test_headers_remove_missing() -> Bool {
  var h = HttpHeaders.new();
  HttpHeaders.add(&mut h, "X", "1");
  let removed = HttpHeaders.remove(&mut h, "Y");
  return !removed && HttpHeaders.count(&h) == 1;
}

fn test_request_new_get() -> Bool {
  let req = HttpRequest.new(HttpMethod.GET, "/api/users");
  return req.path == "/api/users";
}

fn test_request_set_get_header() -> Bool {
  var req = HttpRequest.new(HttpMethod.POST, "/submit");
  HttpRequest.set_header(&mut req, "Content-Type", "text/plain");
  let val = HttpRequest.get_header(&req, "Content-Type");
  if val.is_some() { return val.unwrap() == "text/plain"; }
  return false;
}

fn test_request_set_body() -> Bool {
  var req = HttpRequest.new(HttpMethod.PUT, "/data");
  HttpRequest.set_body(&mut req, "hello world");
  return req.body == "hello world";
}

fn test_request_has_header() -> Bool {
  var req = HttpRequest.new(HttpMethod.GET, "/");
  HttpRequest.set_header(&mut req, "Accept", "application/json");
  return HttpRequest.has_header(&req, "Accept") && !HttpRequest.has_header(&req, "Content-Type");
}

fn test_response_new_200() -> Bool {
  let res = HttpResponse.new(200);
  return res.status == 200;
}

fn test_response_set_header_and_body() -> Bool {
  var res = HttpResponse.new(404);
  HttpResponse.set_header(&mut res, "Server", "XIOM");
  HttpResponse.set_body(&mut res, "Not Found");
  let server = HttpResponse.get_header(&res, "Server");
  if server.is_some() {
    return server.unwrap() == "XIOM" && res.body == "Not Found";
  }
  return false;
}

fn test_http_method_to_str() -> Bool {
  let get = HttpMethod.to_str(HttpMethod.GET);
  let post = HttpMethod.to_str(HttpMethod.POST);
  let del = HttpMethod.to_str(HttpMethod.DELETE);
  return get == "GET" && post == "POST" && del == "DELETE";
}

fn test_http_method_from_str_valid() -> Bool {
  let m = HttpMethod.from_str("PUT");
  if m.is_some() {
    match m.unwrap() {
      PUT => { return true; }
      _ => { return false; }
    }
  }
  return false;
}

fn test_http_method_from_str_invalid() -> Bool {
  let m = HttpMethod.from_str("INVALID");
  return m.is_none();
}

fn test_http_method_from_str_all() -> Bool {
  var methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
  var all_ok = true;
  var i = 0;
  while i < 7 {
    let m = HttpMethod.from_str(methods[i]);
    if m.is_none() { all_ok = false; }
    i = i + 1;
  }
  return all_ok;
}

fn test_full_request_response_cycle() -> Bool {
  var req = HttpRequest.new(HttpMethod.POST, "/echo");
  HttpRequest.set_header(&mut req, "Content-Type", "application/json");
  HttpRequest.set_body(&mut req, "{\"msg\":\"hello\"}");

  var res = HttpResponse.new(201);
  HttpResponse.set_header(&mut res, "Content-Type", "application/json");
  HttpResponse.set_body(&mut res, "{\"status\":\"created\"}");

  if req.method.to_str() != "POST" { return false; }
  if res.status != 201 { return false; }
  if req.body != "{\"msg\":\"hello\"}" { return false; }
  if res.body != "{\"status\":\"created\"}" { return false; }

  let req_ct = HttpRequest.get_header(&req, "Content-Type");
  let res_ct = HttpResponse.get_header(&res, "Content-Type");
  if req_ct.is_some() && res_ct.is_some() {
    return req_ct.unwrap() == "application/json" && res_ct.unwrap() == "application/json";
  }
  return false;
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Int {
  var passed = 0;
  var total = 0;

  total = total + 1;
  if test_headers_new_empty() { passed = passed + 1; }

  total = total + 1;
  if test_headers_add_and_get() { passed = passed + 1; }

  total = total + 1;
  if test_headers_add_multiple() { passed = passed + 1; }

  total = total + 1;
  if test_headers_get_missing() { passed = passed + 1; }

  total = total + 1;
  if test_headers_has() { passed = passed + 1; }

  total = total + 1;
  if test_headers_remove() { passed = passed + 1; }

  total = total + 1;
  if test_headers_remove_missing() { passed = passed + 1; }

  total = total + 1;
  if test_request_new_get() { passed = passed + 1; }

  total = total + 1;
  if test_request_set_get_header() { passed = passed + 1; }

  total = total + 1;
  if test_request_set_body() { passed = passed + 1; }

  total = total + 1;
  if test_request_has_header() { passed = passed + 1; }

  total = total + 1;
  if test_response_new_200() { passed = passed + 1; }

  total = total + 1;
  if test_response_set_header_and_body() { passed = passed + 1; }

  total = total + 1;
  if test_http_method_to_str() { passed = passed + 1; }

  total = total + 1;
  if test_http_method_from_str_valid() { passed = passed + 1; }

  total = total + 1;
  if test_http_method_from_str_invalid() { passed = passed + 1; }

  total = total + 1;
  if test_http_method_from_str_all() { passed = passed + 1; }

  total = total + 1;
  if test_full_request_response_cycle() { passed = passed + 1; }

  if passed == total { return 0; }
  return 1;
}
