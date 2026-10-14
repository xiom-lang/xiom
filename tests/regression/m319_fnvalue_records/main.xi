// m319 (REL-20261013-2238-engine-2, fnvalue_records): the fn-value ABI
// could not carry by-value aggregate parameters across modules. A provider
// struct (CaptureProvider) declared in module A, built in module B and
// called through its fn-typed field in module C emitted the field's
// function type with a not-yet-defined struct in ARGUMENT position:
//   %struct.CaptureProvider = type { i8*, %struct.CaptureRun (i8*, i64,
//     %struct.Vec, ...)* }
// while `%struct.Vec` sorted AFTER "CaptureProvider" (alphabetical type
// emission) -- clang: "invalid type for function argument" (no binary).
// The engine saw this as "a CaptureProvider.run field typed with Vec
// parameters fails LLVM codegen"; providers were reduced to scalars.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m319_fnvalue_records.main;

use xiom.io;
use xiom.convert;
use xiom.m319cap.capture;
use xiom.m319cap.gpu;

fn main() -> Int {
  var prov = make_provider();
  var run = capture.cap_run(prov);
  if !run.ok { return 1; }
  if run.frames_done != 6 { return 2; }

  // The provider called from the entry module directly (control).
  var v = Vec[UInt8].new();
  v.push(1);
  var run2 = prov.run("out", 2, v, v, v, v);
  if run2.frames_done != 6 { return 3; }

  // A record-typed provider control: by-value struct params on a fn value.
  var scaled = capture.scale_point(7);
  if scaled != 21 { return 4; }

  io.println("m319_fnvalue_records: frames=" + int_to_string(run.frames_done));
  return 0;
}
