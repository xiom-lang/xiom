// Phase 3 method gate, accept case: unique-candidate method capture
// (`PathBuf.join` resolves to the sole `Path.join` receiver fn).
use xiom.path;

fn main() -> Int {
  var p = path.Path.new("/home/u");
  var j = p.join("a").join("b");
  return 0;
}
