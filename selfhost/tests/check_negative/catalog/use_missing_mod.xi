// Phase 3 catalog gate, accept case: a `use` of a module that does not
// exist is not an error in the Rust checker (permissive fallback).
use nosuchmodule_xyz;

fn main() -> Int {
  return 0;
}
