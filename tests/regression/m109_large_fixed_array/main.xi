// R54 lock (p_sweep_single_param clang ISel crash): a LARGE fixed array
// (`[65536]UInt8`) must not materialize whole-aggregate values. clang 22.1.8
// X86 ISel crashes on `alloca [65536 x i8]` + `store zeroinitializer` / whole
// array loads, so codegen byte-zeros big arrays with memset and accesses them
// through their address.

fn main() -> Int {
  var buf: [65536]UInt8;
  buf[0] = 42;
  buf[65535] = 7;
  if buf[0] as Int != 42 { return 1; }
  if buf[65535] as Int != 7 { return 2; }
  if buf[1] as Int != 0 { return 3; }

  var mid: [32768]Int;   // nested-type large array, still above the threshold
  mid[0] = 1000000;
  mid[32767] = 2000000;
  if mid[0] != 1000000 { return 4; }
  if mid[32767] != 2000000 { return 5; }
  return 0;
}
