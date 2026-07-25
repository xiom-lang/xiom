// XIOM — Ecosystem JSON Type Hardening Tests
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// Self-contained JSON type definitions and tests. Exercises enums,
// structs with Vec fields, Option, pattern matching, and recursive types.

module tests.ecosystem.test_json

pub enum JsonValue {
  Null,
  Bool(val: Bool),
  Number(val: Float64),
  String(val: Str),
  Array(val: Vec[JsonValue]),
  Object(val: Vec[JsonEntry]),
}

pub type JsonEntry = {
  key: Str;
  value: JsonValue;
}

pub type ParseError = {
  message: Str;
  pos: Int;
}

fn json_null() -> JsonValue {
  return JsonValue.Null;
}

fn json_bool(val: Bool) -> JsonValue {
  return JsonValue.Bool(val);
}

fn json_number(val: Float64) -> JsonValue {
  return JsonValue.Number(val);
}

fn json_string(val: Str) -> JsonValue {
  return JsonValue.String(val);
}

fn json_array() -> JsonValue {
  var arr = Vec[JsonValue].new();
  return JsonValue.Array(arr);
}

fn json_object() -> JsonValue {
  var entries = Vec[JsonEntry].new();
  return JsonValue.Object(entries);
}

fn JsonValue.is_null() -> Bool {
  match this {
    Null => { return true; }
    _ => { return false; }
  }
}

fn JsonValue.is_bool() -> Bool {
  match this {
    Bool(_) => { return true; }
    _ => { return false; }
  }
}

fn JsonValue.is_number() -> Bool {
  match this {
    Number(_) => { return true; }
    _ => { return false; }
  }
}

fn JsonValue.is_string() -> Bool {
  match this {
    String(_) => { return true; }
    _ => { return false; }
  }
}

fn JsonValue.is_array() -> Bool {
  match this {
    Array(_) => { return true; }
    _ => { return false; }
  }
}

fn JsonValue.is_object() -> Bool {
  match this {
    Object(_) => { return true; }
    _ => { return false; }
  }
}

fn JsonValue.as_bool() -> Option[Bool] {
  match this {
    Bool(val) => { return Some(val); }
    _ => { return None; }
  }
}

fn JsonValue.as_number() -> Option[Float64] {
  match this {
    Number(val) => { return Some(val); }
    _ => { return None; }
  }
}

fn JsonValue.as_string() -> Option[Str] {
  match this {
    String(val) => { return Some(val); }
    _ => { return None; }
  }
}

fn json_array_push(arr: &mut JsonValue, val: JsonValue) -> Bool {
  match arr {
    JsonValue.Array(ref mut items) => {
      items.push(val);
      return true;
    }
    _ => { return false; }
  }
}

fn json_array_len(arr: &JsonValue) -> Int {
  match arr {
    JsonValue.Array(ref items) => { return items.len(); }
    _ => { return 0; }
  }
}

fn json_object_insert(obj: &mut JsonValue, key: Str, val: JsonValue) -> Bool {
  match obj {
    JsonValue.Object(ref mut entries) => {
      var entry = JsonEntry{ key: key, value: val };
      entries.push(entry);
      return true;
    }
    _ => { return false; }
  }
}

fn json_get(obj: &JsonValue, key: Str) -> Option[JsonValue] {
  match obj {
    JsonValue.Object(ref entries) => {
      var i = 0;
      while i < entries.len() {
        if entries[i].key == key {
          return Some(entries[i].value);
        }
        i = i + 1;
      }
      return None;
    }
    _ => { return None; }
  }
}

fn json_object_has_key(obj: &JsonValue, key: Str) -> Bool {
  match obj {
    JsonValue.Object(ref entries) => {
      var i = 0;
      while i < entries.len() {
        if entries[i].key == key {
          return true;
        }
        i = i + 1;
      }
      return false;
    }
    _ => { return false; }
  }
}

fn json_stringify(val: &JsonValue) -> Str {
  match val {
    JsonValue.Null => { return "null"; }
    JsonValue.Bool(b) => {
      if b { return "true"; }
      return "false";
    }
    JsonValue.Number(n) => {
      return float_to_string(n);
    }
    JsonValue.String(s) => {
      return "\"" + s + "\"";
    }
    JsonValue.Array(ref items) => {
      var result = "[";
      var i = 0;
      while i < items.len() {
        if i > 0 { result = result + ", "; }
        result = result + json_stringify(&items[i]);
        i = i + 1;
      }
      result = result + "]";
      return result;
    }
    JsonValue.Object(ref entries) => {
      var result = "{";
      var i = 0;
      while i < entries.len() {
        if i > 0 { result = result + ", "; }
        result = result + "\"" + entries[i].key + "\": ";
        result = result + json_stringify(&entries[i].value);
        i = i + 1;
      }
      result = result + "}";
      return result;
    }
  }
}

fn float_to_string(n: Float64) -> Str {
  var whole = float_to_int(n);
  var frac = float_to_int((n - int_to_float(whole)) * 1000000.0);
  if frac < 0 { frac = -frac; }
  // Clone `whole` into a temp to avoid the checker's "use of moved value"
  // on Int (scalars are implicitly Copy, but the checker flags the second
  // use after a conditional return).
  var whole_str = int_to_str(whole);
  if frac == 0 { return whole_str; }
  return whole_str + "." + int_to_str(frac);
}

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

fn float_to_int(n: Float64) -> Int {
  var trunc: Int = 0;
  if n < 0.0 {
    var abs = -n;
    trunc = int_from_float(abs);
    return -trunc;
  }
  trunc = int_from_float(n);
  return trunc;
}

fn int_from_float(n: Float64) -> Int {
  if n >= 1.0 {
    var i = 0;
    while n >= 1.0 {
      n = n - 1.0;
      i = i + 1;
    }
    return i;
  }
  return 0;
}

fn int_to_float(n: Int) -> Float64 {
  var result = 0.0;
  var remaining = n;
  if remaining < 0 {
    while remaining < 0 { result = result - 1.0; remaining = remaining + 1; }
    return result;
  }
  while remaining > 0 { result = result + 1.0; remaining = remaining - 1; }
  return result;
}

// ============================================================================
// Tests
// ============================================================================

fn test_json_null_constructor() -> Bool {
  let v = json_null();
  return v.is_null();
}

fn test_json_bool_constructor_true() -> Bool {
  let v = json_bool(true);
  return v.is_bool() && v.as_bool().unwrap();
}

fn test_json_bool_constructor_false() -> Bool {
  let v = json_bool(false);
  return v.is_bool() && !v.as_bool().unwrap();
}

fn test_json_number_constructor() -> Bool {
  let v = json_number(42.5);
  return v.is_number();
}

fn test_json_number_as_number() -> Bool {
  let v = json_number(3.14);
  let opt = v.as_number();
  if opt.is_some() { return opt.unwrap() == 3.14; }
  return false;
}

fn test_json_string_constructor() -> Bool {
  let v = json_string("hello");
  return v.is_string();
}

fn test_json_string_as_string() -> Bool {
  let v = json_string("world");
  let opt = v.as_string();
  if opt.is_some() { return opt.unwrap() == "world"; }
  return false;
}

fn test_json_array_constructor() -> Bool {
  let v = json_array();
  return v.is_array();
}

fn test_json_object_constructor() -> Bool {
  let v = json_object();
  return v.is_object();
}

fn test_is_null_on_non_null() -> Bool {
  let v = json_bool(true);
  return !v.is_null();
}

fn test_is_number_on_string() -> Bool {
  let v = json_string("test");
  return !v.is_number();
}

fn test_json_array_push_and_len() -> Bool {
  var arr = json_array();
  json_array_push(&mut arr, json_number(1.0));
  json_array_push(&mut arr, json_number(2.0));
  json_array_push(&mut arr, json_number(3.0));
  return json_array_len(&arr) == 3;
}

fn test_json_array_push_on_non_array() -> Bool {
  var v = json_null();
  let ok = json_array_push(&mut v, json_number(1.0));
  return !ok;
}

fn test_json_object_insert_and_get() -> Bool {
  var obj = json_object();
  json_object_insert(&mut obj, "name", json_string("xiom"));
  json_object_insert(&mut obj, "version", json_number(1.0));
  let name = json_get(&obj, "name");
  let version = json_get(&obj, "version");
  let missing = json_get(&obj, "missing");
  if name.is_some() && version.is_some() && missing.is_none() {
    return name.unwrap().as_string().unwrap() == "xiom";
  }
  return false;
}

fn test_json_object_has_key() -> Bool {
  var obj = json_object();
  json_object_insert(&mut obj, "key", json_bool(true));
  let has = json_object_has_key(&obj, "key");
  let missing = json_object_has_key(&obj, "nope");
  return has && !missing;
}

fn test_json_get_on_non_object() -> Bool {
  let v = json_null();
  let result = json_get(&v, "key");
  return result.is_none();
}

fn test_json_stringify_null() -> Bool {
  let v = json_null();
  return json_stringify(&v) == "null";
}

fn test_json_stringify_bool_true() -> Bool {
  let v = json_bool(true);
  return json_stringify(&v) == "true";
}

fn test_json_stringify_bool_false() -> Bool {
  let v = json_bool(false);
  return json_stringify(&v) == "false";
}

fn test_json_stringify_number() -> Bool {
  let v = json_number(7.0);
  let s = json_stringify(&v);
  return s == "7";
}

fn test_json_stringify_string() -> Bool {
  let v = json_string("xiom");
  return json_stringify(&v) == "\"xiom\"";
}

fn test_json_stringify_empty_array() -> Bool {
  let v = json_array();
  return json_stringify(&v) == "[]";
}

fn test_json_stringify_array_with_items() -> Bool {
  var arr = json_array();
  json_array_push(&mut arr, json_number(1.0));
  json_array_push(&mut arr, json_number(2.0));
  let s = json_stringify(&arr);
  return s == "[1, 2]";
}

fn test_json_stringify_empty_object() -> Bool {
  let v = json_object();
  return json_stringify(&v) == "{}";
}

fn test_json_as_bool_on_non_bool() -> Bool {
  let v = json_null();
  return v.as_bool().is_none();
}

fn test_json_as_number_on_non_number() -> Bool {
  let v = json_string("x");
  return v.as_number().is_none();
}

fn test_json_as_string_on_non_string() -> Bool {
  let v = json_number(10.0);
  return v.as_string().is_none();
}

fn test_json_nested_object() -> Bool {
  var inner = json_object();
  json_object_insert(&mut inner, "x", json_number(1.0));
  var outer = json_object();
  json_object_insert(&mut outer, "inner", inner);
  let retrieved = json_get(&outer, "inner");
  if retrieved.is_some() {
    let inner_val = json_get(&retrieved.unwrap(), "x");
    if inner_val.is_some() {
      return inner_val.unwrap().as_number().unwrap() == 1.0;
    }
  }
  return false;
}

fn test_json_nested_array() -> Bool {
  var inner = json_array();
  json_array_push(&mut inner, json_number(10.0));
  json_array_push(&mut inner, json_number(20.0));
  var outer = json_array();
  json_array_push(&mut outer, inner);
  return json_array_len(&outer) == 1;
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Int {
  var passed = 0;
  var total = 0;

  total = total + 1;
  if test_json_null_constructor() { passed = passed + 1; }

  total = total + 1;
  if test_json_bool_constructor_true() { passed = passed + 1; }

  total = total + 1;
  if test_json_bool_constructor_false() { passed = passed + 1; }

  total = total + 1;
  if test_json_number_constructor() { passed = passed + 1; }

  total = total + 1;
  if test_json_number_as_number() { passed = passed + 1; }

  total = total + 1;
  if test_json_string_constructor() { passed = passed + 1; }

  total = total + 1;
  if test_json_string_as_string() { passed = passed + 1; }

  total = total + 1;
  if test_json_array_constructor() { passed = passed + 1; }

  total = total + 1;
  if test_json_object_constructor() { passed = passed + 1; }

  total = total + 1;
  if test_is_null_on_non_null() { passed = passed + 1; }

  total = total + 1;
  if test_is_number_on_string() { passed = passed + 1; }

  total = total + 1;
  if test_json_array_push_and_len() { passed = passed + 1; }

  total = total + 1;
  if test_json_array_push_on_non_array() { passed = passed + 1; }

  total = total + 1;
  if test_json_object_insert_and_get() { passed = passed + 1; }

  total = total + 1;
  if test_json_object_has_key() { passed = passed + 1; }

  total = total + 1;
  if test_json_get_on_non_object() { passed = passed + 1; }

  total = total + 1;
  if test_json_stringify_null() { passed = passed + 1; }

  total = total + 1;
  if test_json_stringify_bool_true() { passed = passed + 1; }

  total = total + 1;
  if test_json_stringify_bool_false() { passed = passed + 1; }

  total = total + 1;
  if test_json_stringify_number() { passed = passed + 1; }

  total = total + 1;
  if test_json_stringify_string() { passed = passed + 1; }

  total = total + 1;
  if test_json_stringify_empty_array() { passed = passed + 1; }

  total = total + 1;
  if test_json_stringify_array_with_items() { passed = passed + 1; }

  total = total + 1;
  if test_json_stringify_empty_object() { passed = passed + 1; }

  total = total + 1;
  if test_json_as_bool_on_non_bool() { passed = passed + 1; }

  total = total + 1;
  if test_json_as_number_on_non_number() { passed = passed + 1; }

  total = total + 1;
  if test_json_as_string_on_non_string() { passed = passed + 1; }

  total = total + 1;
  if test_json_nested_object() { passed = passed + 1; }

  total = total + 1;
  if test_json_nested_array() { passed = passed + 1; }

  if passed == total { return 0; }
  return 1;
}
