// Spawn Tests — v0.55
// Verifies spawn { ... } compiles and the thread runtime links.
// Returns 0 on success.

fn main() -> Int {
  // Basic spawn — creates an OS thread via xiom_thread_spawn
  spawn {
    var x: Int = 42;
  }

  // Multiple spawns
  spawn { var y: Int = 1; }
  spawn { var z: Int = 2; }

  return 0;
}
