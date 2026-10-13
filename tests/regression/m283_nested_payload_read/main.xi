// m283 lock (REL-20261013-1524-bindings-2 B-26): a nested enum payload
// (`Result[Option[Bag], Str]`) must read back the concrete payload struct,
// not the erased base `%struct.Option` (`{ i64, i64 }`). Pre-fix the payload
// read loaded 16 bytes of the heap box as an inline struct: the tag picked up
// pointer bits and `items.len()` read heap garbage (observed 9), then using
// the corrupt Vec AV'd. Root cause: `llvm_type_for("Option[Bag]")` stripped
// the generic args and returned the unspecialized `%struct.Option` even when
// the concrete `%struct.Option__Bag` monomorph was registered. Controls:
// flat `Result[BagOutcome, Str]` and single-level `Option[Bag]` were green.
// Pure XIOM -- no C source. Copyright (c) 2026 Eleftherios Notas and The XIOM
// Authors -- SPDX-License-Identifier: MIT OR Apache-2.0
module m283_nested_payload_read

use xiom.io;
use xiom.convert;

pub type Bag = {
  items: Vec[Int];
}

pub type BagOutcome = {
  has: Bool;
  bag: Bag;
}

fn bag_new() -> Bag {
  var b = Bag{ items: Vec[Int].new() };
  b.items.push(7);
  b.items.push(9);
  return b;
}

// Nested shape: enum payload inside enum payload.
fn nested_ok() -> Result[Option[Bag], Str] {
  return Ok(Some(bag_new()));
}

// Flat control: plain struct payload.
fn flat_ok() -> Result[BagOutcome, Str] {
  return Ok(BagOutcome{ has: true, bag: bag_new() });
}

// Single-level control: Option payload, not nested.
fn opt_ok() -> Option[Bag] {
  return Some(bag_new());
}

fn main() -> Int {
  var bad: Int = 0;

  let r = nested_ok();
  if !r.is_ok {
    io.println("nested: Err (unexpected)");
    bad = bad + 1;
  } else {
    let opt = r.value;
    if !opt.is_some {
      io.println("nested: None (unexpected)");
      bad = bad + 1;
    } else {
      let bag: Bag = opt.value;
      let n = bag.items.len();
      io.println("nested items.len=" + convert.int_to_string(n) + " (expected 2)");
      if n != 2 { bad = bad + 1; }
      // using the value must not AV
      if n == 2 {
        let first = bag.items[0];
        if first != 7 { bad = bad + 1; }
      }
    }
  }

  let f = flat_ok();
  if !f.is_ok {
    io.println("flat: Err (unexpected)");
    bad = bad + 1;
  } else {
    let out: BagOutcome = f.value;
    let nf = out.bag.items.len();
    io.println("flat items.len=" + convert.int_to_string(nf) + " (expected 2)");
    if nf != 2 { bad = bad + 1; }
  }

  let o = opt_ok();
  if !o.is_some {
    io.println("opt: None (unexpected)");
    bad = bad + 1;
  } else {
    let bag2: Bag = o.value;
    let no = bag2.items.len();
    io.println("opt items.len=" + convert.int_to_string(no) + " (expected 2)");
    if no != 2 { bad = bad + 1; }
  }

  io.println("failures=" + convert.int_to_string(bad));
  return bad;
}
