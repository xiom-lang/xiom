// m80 (R23): fn-typed VALUES are closure ENV pointers on the uniform
// env-first convention (wrap_fn_ref_env / M20-A1). Every call-through-value
// shape must load the trampoline from env[0] and pass the env as the first
// argument. Pre-fix, `callee_is_fn_ptr` (and struct-field stores) treated
// the env box as CODE -> 0xC0000005; the async executor's reduced shapes
// AV'd while the full smoke surface happened to pass (shape dependence).
module m80_fn_value_shapes

use xiom.io;

fn add1(x: Int) -> Int { return x + 1; }

// (a) fn-typed PARAM
fn apply(f: fn(Int) -> Int, x: Int) -> Int { return f(x); }

// (c) fn-typed STRUCT FIELD
type FnBox = { f: fn(Int) -> Int; }

// (d) fn-typed VEC ELEMENT + pop payload binding (the executor shape)
type TaskBox = { ready: Vec[fn()]; }
fn tick() -> Int { return 7; }

fn step(b: &mut TaskBox) -> Int {
  match b.ready.pop() {
    Some(task) => { task(); return 1; },
    None => { return 0; },
  }
}

fn main() -> Int {
  // (a)
  if apply(add1, 41) != 42 { return 1; }
  // (b) local binding of a bare fn ref
  let g = add1;
  if g(41) != 42 { return 2; }
  // (c) struct field
  var b = FnBox{ f: add1 };
  if b.f(41) != 42 { return 3; }
  // (d) vec element push + pop-bound call
  var tb = TaskBox{ ready: Vec[fn()].new() };
  tb.ready.push(tick);
  if tb.ready.len() != 1 { return 4; }
  if step(&mut tb) != 1 { return 5; }
  if tb.ready.len() != 0 { return 6; }
  io.println("M80 OK");
  return 0;
}
