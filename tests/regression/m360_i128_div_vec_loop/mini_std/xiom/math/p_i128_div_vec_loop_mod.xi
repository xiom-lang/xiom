// m360 (REL-20261016-2331-stdlib) support module for the regression test.
// Lives under the fixture's OWN mini XIOM_STDLIB so the import resolves as
// xiom.math.p_i128_div_vec_loop_mod (the witness requires an xiom.* module).
//
// A vector element-load loop followed by one RUNTIME 128-bit division in the
// SAME function. Pre-fix (compiler v0.64.7) the loop kept LLVM from folding
// the divide, so the emitted sdiv i128 reached the stdlib runtime's recursive
// __udivti3 helper and the process AV'd with 0xC0000005. Expected: 2.
module xiom.math.p_i128_div_vec_loop_mod

/// A vector element-load loop followed by one runtime 128-bit division in
/// the SAME function. Pre-fix this compiles (rc 0) and the caller AVs with
/// 0xC0000005 at run time on the pinned compiler.
pub fn ta4(values: &Vec[Int], scale: Int) -> Int {
  var acc = 0;
  var i = 0;
  var n = values.len();
  while i < n {
    acc = acc + values[i];
    i = i + 1;
  }
  var a: Int128 = acc as Int128;
  var s: Int128 = scale as Int128;
  var q: Int128 = (a + a + s) / (s + s);
  return q as Int;
}
