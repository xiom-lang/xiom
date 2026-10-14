// m319 (REL-20261013-2238-engine-2): the CaptureProvider shape -- a fn-typed
// field carrying by-value Vec parameters and a by-value struct return, all
// declared in THIS module and resolved by the other modules.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.m319cap.capture;

pub type CaptureRun = {
  ok: Bool;
  skip: Bool;
  reason: Str;
  frames_done: Int;
  frame_ms: Vec[Float64];
}

pub type CaptureProvider = {
  name: Str;
  run: fn(Str, Int, Vec[UInt8], Vec[UInt8], Vec[UInt8], Vec[UInt8]) -> CaptureRun;
}

pub type Point = {
  x: Int;
  y: Int;
}

/// By-value record arg through a fn value (the item's "fnvalue_records").
pub type Scaler = {
  run: fn(Point) -> Int;
}

fn scale(p: Point) -> Int { return p.x * p.y; }

pub fn scale_point(n: Int) -> Int {
  var s = Scaler{ run: scale };
  return s.run(Point{ x: n; y: 3 });
}

pub fn cap_run(provider: CaptureProvider) -> CaptureRun {
  var v = Vec[UInt8].new();
  v.push(1);
  return provider.run("out", 2, v, v, v, v);
}
