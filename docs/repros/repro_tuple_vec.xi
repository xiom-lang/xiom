// REPRO R5 (BUG 27 #10): tuple-with-Vec returns heap-corrupt (0xC0000374).
// aes_encrypt_gcm returns Result[(Vec[UInt8], Vec[UInt8]), Str] -- the tuple
// payload with Vec elements corrupts memory at the catalog boundary.
module repro_tuple_vec

pub fn gcm_like(a: Vec[UInt8], b: Vec[UInt8]) -> Result[(Vec[UInt8], Vec[UInt8]), Str] {
  return Ok((a, b));
}