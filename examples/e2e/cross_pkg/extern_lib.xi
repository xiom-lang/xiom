module extern_lib
extern "C" {
  fn puts(s: *UInt8) -> Int;
}
pub fn greet() -> Int {
  return puts("hello");
}
