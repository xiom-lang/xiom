// R8 follow-up regression: Str method-parity sugar on a Str PARAM.
// `.trim()` / `.trim_start()` / `.trim_end()` in method position returned a
// corrupt Str (len=0xFFFFFFFF) because codegen auto-stubbed `Str.trim` and
// the reachability filter had pruned the canonical free fns. Method sugar is
// now routed to the canonical stdlib fns and their bodies are seeded into
// the injection.
module m65_str_method_sugar

use xiom.io;

fn trimmed(s: Str) -> Str {
  return s.trim();
}

fn trimmed_start(s: Str) -> Str {
  return s.trim_start();
}

fn trimmed_end(s: Str) -> Str {
  return s.trim_end();
}

fn main() -> Int {
  let a = trimmed("  hi  ");
  if a.len() != 2 { return 1; }
  if a != "hi" { return 2; }
  let b = trimmed_start("  hi  ");
  if b != "hi  " { return 3; }
  let c = trimmed_end("  hi  ");
  if c != "  hi" { return 4; }
  return 0;
}
