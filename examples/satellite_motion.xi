// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM -- Satellite Motion + Trig Verification
// Uses cos/sin from xiom.math. Pure math checks, no string formatting.
// Compile: xiom examples\satellite_motion.xi --run

module satellite_motion
use xiom.math;

const GM: Float64 = 3.986e14;
const EARTH_RADIUS: Float64 = 6371000.0;
const ALTITUDE: Float64 = 408000.0;
const R: Float64 = 6779000.0; // EARTH_RADIUS + ALTITUDE (const arithmetic gap -- pre-computed)

fn main() -> Int {
  var fail = 0;

  // Test 1: Circular orbit period
  var period = 2.0 * math.PI * math.sqrt(R * R * R / GM);
  // ISS period is ~5560 seconds (92.7 min)
  if period < 5000.0 || period > 6000.0 { fail = fail + 1; }

  // Test 2: Velocity
  var v = math.sqrt(GM / R);
  if v < 7500.0 || v > 7800.0 { fail = fail + 2; }

  // Test 3: cos(0) = 1
  var c0 = math.cos(0.0);
  if math.abs_float(c0 - 1.0) > 0.001 { fail = fail + 4; }

  // Test 4: sin(0) = 0
  var s0 = math.sin(0.0);
  if math.abs_float(s0 - 0.0) > 0.001 { fail = fail + 8; }

  // Test 5: cos(PI) = -1
  var cp = math.cos(math.PI);
  if math.abs_float(cp + 1.0) > 0.01 { fail = fail + 16; }

  // Test 6: sin(PI/2) = 1
  var sp2 = math.sin(math.PI / 2.0);
  if math.abs_float(sp2 - 1.0) > 0.01 { fail = fail + 32; }

  // Test 7: sin2 + cos2 = 1
  var angle = 0.7;
  var s = math.sin(angle);
  var c = math.cos(angle);
  var sq = s * s + c * c;
  if math.abs_float(sq - 1.0) > 0.001 { fail = fail + 64; }

  // Test 8: tan = sin / cos
  var t = math.tan(0.5);
  var ratio = math.sin(0.5) / math.cos(0.5);
  if math.abs_float(t - ratio) > 0.002 { fail = fail + 128; }

  // Test 9: Pure XIOM fallback agrees with FFI
  var sp = math.sin_pure(0.5);
  var sf = math.sin(0.5);
  if math.abs_float(sp - sf) > 0.002 { fail = fail + 256; }

  // Test 10: Satellite positions at t=0 and t=T/4
  var omega = 2.0 * math.PI / period;
  var x0 = R * math.cos(0.0);         // t=0: should be at (R, 0)
  var y0 = R * math.sin(0.0);
  var x1 = R * math.cos(omega * period / 4.0); // t=T/4: should be at (0, R)
  var y1 = R * math.sin(omega * period / 4.0);

  if math.abs_float(x0 - R) > 500.0 { fail = fail + 512; }
  if math.abs_float(y0) > 500.0 { fail = fail + 1024; }
  if math.abs_float(x1) > 5000.0 { fail = fail + 2048; }
  if math.abs_float(y1 - R) > 5000.0 { fail = fail + 4096; }

  return fail;
}
