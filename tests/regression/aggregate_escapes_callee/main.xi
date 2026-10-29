// REL-20261029-2110-packages: an aggregate local with an owned Vec field that
// is passed BY VALUE to a callee which stores it (sim_send_a -> _enqueue ->
// q_ab) must not be dropped by the caller. The callee/container owns the
// shallow-copied field header; the caller's exit drop freed the payload
// (ASan UAF in sim_send_a; sim r2 became [k3,k2,k3], k0=0).
module aggregate_escapes_callee

use xiom.io;
use xiom.convert;

type Packet = { id: Int; bytes: Vec[UInt8]; }
type Link = { q: Vec[Packet]; cap: Vec[Packet]; }

fn enqueue(l: &mut Link, p: Packet) {
  l.q.push(p);
}

// Borrowed by a callee that retains its own copy; must not change the
// caller's ownership (the packet is still moved into enqueue afterwards).
fn capture(l: &mut Link, p: &Packet) {
  var c = Packet{ id: p.id, bytes: Vec[UInt8].new() };
  c.bytes.push(99);
  l.cap.push(c);
}

fn send_packet(l: &mut Link, id: Int, byte: UInt8) {
  var p = Packet{ id: id, bytes: Vec[UInt8].new() };
  p.bytes.push(byte);
  p.bytes.push(byte);
  capture(l, &p);
  enqueue(l, p);
}

fn drain(l: &mut Link) -> Vec[Vec[UInt8]] {
  var out = Vec[Vec[UInt8]].new();
  var i = 0;
  while i < l.q.len() {
    let p = l.q[i];
    out.push(p.bytes);
    i = i + 1;
  }
  l.q = Vec[Packet].new();
  return out;
}

fn churn() -> Int {
  var junk = Vec[Vec[UInt8]].new();
  var i = 0;
  while i < 8000 {
    var v = Vec[UInt8].new();
    v.push(85);
    junk.push(v);
    i = i + 1;
  }
  return junk.len();
}

fn main() -> Int {
  var round = 0;
  while round < 20 {
    var l = Link{ q: Vec[Packet].new(), cap: Vec[Packet].new() };
    send_packet(&mut l, 0, 65 as UInt8);
    send_packet(&mut l, 1, 66 as UInt8);
    send_packet(&mut l, 2, 67 as UInt8);
    let got = drain(&mut l);
    let _j = churn();
    if got.len() != 3 { return 1; }
    if got[0].len() != 2 { return 2; }
    if got[0][0] != 65 as UInt8 { return 3; }
    if got[1][0] != 66 as UInt8 { return 4; }
    if got[2][0] != 67 as UInt8 { return 5; }
    if l.cap.len() != 3 { return 6; }
    round = round + 1;
  }
  io.println("aggregate escapes callee ok rounds=" + convert.int_to_string(round));
  return 0;
}
