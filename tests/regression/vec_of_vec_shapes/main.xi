// REL-20261028: Vec[Vec] safety shapes. The element tag must free inner
// buffers only for push-only containers of moved fresh inner Vecs; aliases,
// duplicate pushes, whole-element reads, pop and element mutation must stay
// leak-only (no double free).
module vec_of_vec_shapes

use xiom.io;

fn inline_ok() -> Int {
  var os = Vec[Vec[UInt8]].new();
  os.push(Vec[UInt8].with_len(1024));
  if os[0].len() != 1024 { return 1; }
  return 0;
}

fn alias_after() -> Int {
  var os = Vec[Vec[UInt8]].new();
  var inner = Vec[UInt8].with_len(1024);
  os.push(inner);
  let t = inner;
  if t.len() != 1024 { return 2; }
  return 0;
}

fn dup_push() -> Int {
  var os = Vec[Vec[UInt8]].new();
  var inner = Vec[UInt8].with_len(1024);
  os.push(inner);
  os.push(inner);
  if os.len() != 2 { return 3; }
  return 0;
}

fn whole_elem() -> Int {
  var os = Vec[Vec[UInt8]].new();
  var inner = Vec[UInt8].with_len(1024);
  os.push(inner);
  let v = os[0];
  if v.len() != 1024 { return 4; }
  return 0;
}

fn pop_back() -> Int {
  var os = Vec[Vec[UInt8]].new();
  var inner = Vec[UInt8].with_len(1024);
  os.push(inner);
  match os.pop() {
    Some(v) => { if v.len() != 1024 { return 5; } },
    None => { return 5; },
  }
  return 0;
}

fn elem_push() -> Int {
  var os = Vec[Vec[UInt8]].new();
  var inner = Vec[UInt8].with_len(1024);
  os.push(inner);
  os[0].push(7);
  if os[0].len() != 1025 { return 6; }
  return 0;
}

fn with_capacity_ok() -> Int {
  var os = Vec[Vec[UInt8]].with_capacity(4);
  var inner = Vec[UInt8].with_len(1024);
  os.push(inner);
  if os[0].len() != 1024 { return 7; }
  return 0;
}

fn main() -> Int {
  if inline_ok() != 0 { return 10; }
  if alias_after() != 0 { return 11; }
  if dup_push() != 0 { return 12; }
  if whole_elem() != 0 { return 13; }
  if pop_back() != 0 { return 14; }
  if elem_push() != 0 { return 15; }
  if with_capacity_ok() != 0 { return 16; }
  io.println("vec-of-vec shapes ok");
  return 0;
}
