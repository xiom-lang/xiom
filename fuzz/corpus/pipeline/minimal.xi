// fuzz seed: minimal compilable program for the codegen pipeline.
module seed_pipeline
use xiom.io;

fn main() -> Int {
  io.println("seed");
  return 0;
}
