module cross_pkg.extern_main
use extern_lib;
fn main() -> Int {
  return extern_lib.greet();
}
