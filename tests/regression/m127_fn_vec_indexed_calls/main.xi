// m127 (R72): fn references stored as collection ELEMENTS must be closure ENV
// values (field 0 = code pointer), like every other fn-valued position
// (B-007 args, Vec.push, struct fields). Array literals stored the RAW code
// address instead, so an indexed call `fns[i]()` loaded field 0 from the
// function's machine code and took an access violation (packages relay,
// xiom.test.run_all; workaround run_test_at(index)). `compile_array_as_vec`
// now wraps bare fn references into envs with a forwarding thunk, matching
// the call convention.
module m127_fn_vec_indexed_calls

use xiom.iter;

fn ten() -> Int { return 10; }
fn twenty() -> Int { return 20; }
fn thirty() -> Int { return 30; }

// The relayed run_all shape: a fn-vector param, indexed calls in a range loop.
fn run_all(tests: &Vec[fn() -> Int]) -> Int {
  var total = 0;
  for __i in range(0, tests.len()) {
    total = total + tests[__i]();
  }
  return total;
}

fn main() -> Int {
  var fns: Vec[fn() -> Int] = [ten, twenty, thirty];
  if fns[0]() != 10 { return 1; }
  if fns[2]() != 30 { return 2; }
  if run_all(&fns) != 60 { return 3; }

  // Unannotated binding (element type defaults to Int) must wrap too.
  var bare = [ten, twenty];
  if bare[1]() != 20 { return 4; }

  return 0;
}
