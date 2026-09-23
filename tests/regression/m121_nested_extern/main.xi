// m121 (R68): legacy nested FFI declarations. `extern "C" { ... }` inside a
// function body used to fall into the expression parser, which reported the
// confusing "'extern' is a reserved keyword and cannot be used as an
// identifier" (legacy packages, e.g. the audio_beep shape). Extern fns carry
// no body -- they only declare C symbols -- so the block is now hoisted to the
// enclosing module, immediately before the declaration whose body contained
// it. Duplicate declarations (one block per function) must stay mergeable.
module m121_nested_extern

// The hoisted declaration must be visible to this function too.
fn nested_ok() -> Bool {
  extern "C" {
    fn clock() -> Int;
  }
  return unsafe { clock() } >= 0;
}

fn main() -> Int {
  // 1. Nested extern block inside the entry point itself.
  extern "C" {
    fn clock() -> Int;
  }
  var t = unsafe { clock() };
  if t < 0 { return 1; }

  // 2. Another function declaring the SAME extern fn (duplicate must merge).
  if !nested_ok() { return 2; }

  // 3. Nested block inside an if body.
  if true {
    extern "C" {
      fn clock() -> Int;
    }
    if unsafe { clock() } < 0 { return 3; }
  }

  return 0;
}
