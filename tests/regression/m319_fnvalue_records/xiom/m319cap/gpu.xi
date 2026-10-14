// m319 (REL-20261013-2238-engine-2): the CLI-lane provider module -- builds
// the CaptureProvider with a record/Vec-carrying fn value (the shape that
// failed clang codegen before the m319 type-order fix).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.m319cap.gpu;

use xiom.m319cap.capture;

pub fn capgpu_exec(out_dir: Str, frames: Int, gvb: Vec[UInt8], gib: Vec[UInt8], svb: Vec[UInt8], sib: Vec[UInt8]) -> CaptureRun {
  return CaptureRun{
    ok: true;
    skip: false;
    reason: out_dir;
    frames_done: frames + gvb.len() + gib.len() + svb.len() + sib.len();
    frame_ms: Vec[Float64].new();
  };
}

pub fn make_provider() -> CaptureProvider {
  return CaptureProvider{ name: "gpu"; run: capgpu_exec };
}
