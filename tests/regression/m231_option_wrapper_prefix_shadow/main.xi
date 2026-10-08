// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m231 (BINDINGS B-01): the scrutinee-type lookup for `match s.value` matched
// wrapper type keys by loose suffix -- `Option__Value`/`Result__...__Value`
// also end with `Value`, and the FIRST key in random HashMap order won. When
// Option__Value won, the field named `value` resolved to Option's PAYLOAD
// field type (`Value` instead of `Kind`), so these accessors compared the
// wrong discriminant (Integer at index 0 instead of 1) and read a garbage
// payload -- nondeterministically, per compile (7/12 bad builds on the pkg
// repro). The boundary fix keeps the match correct and deterministic.

module m231_option_wrapper_prefix_shadow

pub type Value = {
  value: Kind;
}

pub enum Kind {
  Null,
  Integer(value: Int),
  Text(value: Str),
}

pub fn Value.integer(n: Int) -> Value {
  return Value{ value: Kind.Integer(n) };
}

pub fn Value.text(s: Str) -> Value {
  return Value{ value: Kind.Text(s) };
}

pub fn Value.as_int(v: &Value) -> Option[Int] {
  match v.value {
    Kind.Integer(value) => Some(value),
    _ => None,
  }
}

pub fn Value.as_text(v: &Value) -> Option[Str] {
  match v.value {
    Kind.Text(value) => Some(value),
    _ => None,
  }
}

// Keep Option[Value] concretized in the graph: this is what registers the
// `Option__Value` wrapper key whose loose suffix match poisoned the lookup.
pub fn get(holder: &Vec[Value], i: Int) -> Option[Value] {
  if i >= holder.len() { return None; }
  return Some(holder[i]);
}

fn main() -> Int {
  var xs = Vec[Value].new();
  xs.push(Value.integer(7));
  xs.push(Value.text("hi"));

  let a = Value.as_int(&xs[0]);
  match a {
    None => { return 1; }
    Some(n) => { if n != 7 { return 2; } }
  }

  let t = Value.as_text(&xs[1]);
  match t {
    None => { return 3; }
    Some(s) => { if s != "hi" { return 4; } }
  }

  let o = get(&xs, 0);
  match o {
    None => { return 5; }
    Some(v) => {
      let n = Value.as_int(&v);
      match n {
        None => { return 6; }
        Some(m) => { if m != 7 { return 7; } }
      }
    }
  }
  return 0;
}
