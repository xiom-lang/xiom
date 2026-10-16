// m346 (REL-20261016-1541-xvector / XVC-C-18): an INFERRED local bound from
// `x.field[i]` lost the element type whenever the field's element needed
// GENERIC SUBSTITUTION -- Map's `keys: Vec[K]` registered RAW, so the slot
// degraded to i64 and `.len()` typed as the raw param K.
//
// Pre-fix (base d4fdd7e7):
//   * arm A `var name = entries.keys[i]` degraded to an i64 slot -- the key
//     printed as pointer digits and `name_ok`-style consumers received the
//     slot ADDRESS (a VALID mapping file was refused in router_mapping_load);
//   * `var l = name.len()` was a false error[T001] "cannot call 'len'"
//     (the binding typed as the un-substituted param K) while the same value
//     still coerced into a Str parameter -- silent garbage instead of a type
//     error.
// Post-fix: the binding keeps Str (i8* slot, inttoptr of the handle),
// `.len()` resolves, and the key compares equal to its literal.
//
// Arm B (`scan_param_rooted`) is COMPILE-ONLY: the CHECKER half of the
// param-rooted shape (T001 on `name.len()` inside a fn taking
// `Map[Str, JsonValue]`) is locked here and by the --check arm in
// crates/xiom/tests/checker_locks.rs. The param-rooted VALUES remain G2
// (decl.rs records params with ERASED container names -- "Map" -- so the
// codegen substitution path cannot derive K=Str and direct-chain/copy-out
// consumers inside such fns still lower the key wrongly). The fn is never
// called at runtime for that reason; see docs/COMPILER_BUGS.md (m346 G2).
//
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m346_field_chain_elem_binding;

use xiom.serialize.json;

// c2 control: a CONCRETE struct field Vec[Str] (no substitution needed).
type NameBox = { names: Vec[Str]; }

// c1 control: element read off a `&Vec[Str]` receiver.
fn first_slice(v: &Vec[Str]) -> Str {
  return v[0];
}

// arm B (compile-only, G2-documented): the param-rooted direct chain must
// pass the CHECKER (`name.len()` was T001 pre-fix) -- never called.
fn scan_param_rooted(entries: Map[Str, JsonValue]) -> Int {
  var i: Int = 0;
  var hits: Int = 0;
  while i < entries.keys.len() {
    var name = entries.keys[i];
    if name.len() > 0 { hits = hits + 1; }
    i = i + 1;
  }
  return hits;
}

// arm A: the filed XVC-C-18 payload shape -- match enum binding (`entries`),
// generic-substituted field chain (`entries.keys[i]`), inferred local.
fn scan_payload(doc: JsonValue) -> Int {
  var fails: Int = 0;
  match json.json_get(doc, "collections") {
    Some(cv) => {
      match cv {
        Object(entries) => {
          var i: Int = 0;
          while i < entries.keys.len() {
            var name = entries.keys[i];           // inferred Str binding
            var l = name.len();                   // false T001 pre-fix
            if l != 4 { fails = fails + 1; }      // "docs" / "logs"
            if name != "docs" && name != "logs" { fails = fails + 1; }
            i = i + 1;
          }
        }
        _ => { fails = fails + 1; }
      }
    }
    None => { fails = fails + 1; }
  }
  return fails;
}

fn main() -> Int {
  var fails: Int = 0;

  // arm A
  var p = json.json_parse("{\"collections\":{\"docs\":\"s1\",\"logs\":\"s2\"}}");
  match p {
    Ok(doc) => { fails = fails + scan_payload(doc); }
    Err(_) => { fails = fails + 1; }
  }

  // control: payload direct chain COPY-OUT (`var ks = entries.keys;`) --
  // green before and after (the shipped lane workaround).
  var p2 = json.json_parse("{\"collections\":{\"docs\":\"s1\"}}");
  match p2 {
    Ok(doc2) => {
      match json.json_get(doc2, "collections") {
        Some(cv2) => {
          match cv2 {
            Object(entries2) => {
              var ks = entries2.keys;
              var name2 = ks[0];
              if name2 != "docs" { fails = fails + 1; }
            }
            _ => { fails = fails + 1; }
          }
        }
        None => { fails = fails + 1; }
      }
    }
    Err(_) => { fails = fails + 1; }
  }

  // control: EXPLICIT-typed local (`var name: Str = ...`) -- green before
  // and after; pins that the annotation path keeps working.
  var p3 = json.json_parse("{\"collections\":{\"docs\":\"s1\"}}");
  match p3 {
    Ok(doc3) => {
      match json.json_get(doc3, "collections") {
        Some(cv3) => {
          match cv3 {
            Object(entries3) => {
              var name3: Str = entries3.keys[0];
              if name3 != "docs" { fails = fails + 1; }
            }
            _ => { fails = fails + 1; }
          }
        }
        None => { fails = fails + 1; }
      }
    }
    Err(_) => { fails = fails + 1; }
  }

  // c1 control: &Vec[Str] element
  var vs = Vec[Str].new();
  vs.push("abc");
  var c1 = first_slice(vs);
  if c1 != "abc" { fails = fails + 1; }

  // c2 control: concrete struct field Vec[Str]
  var nb = NameBox{ names: Vec[Str].new() };
  nb.names.push("xyz");
  var c2 = nb.names[0];
  if c2 != "xyz" { fails = fails + 1; }

  return fails;
}
