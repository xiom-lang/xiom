// M36-R28: very large float literal -- parser must handle extreme float values
fn main() -> Int {
  var x: Float64 = 1.7976931348623157e308;
  return 0;
}
