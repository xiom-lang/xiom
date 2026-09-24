// m130 (C1, fn-value ABI): fn-typed elements keep the uniform closure ENV
// representation through every path -- `.new()+push`, element calls, element
// passed to a fn-typed param, fn-typed Vec inside a struct, an unannotated
// fn array (for-in + element-to-local) and closure elements. Before the fix
// the push path stored raw code addresses (AV) and unannotated arrays lost
// the element marker ("element type could not be resolved").
module m130_fn_value_abi;

fn ten() -> Int { return 10; }
fn apply(f: fn() -> Int) -> Int { return f(); }
type Suite = { tests: Vec[fn() -> Int]; }

fn main() -> Int {
  var fs: Vec[fn() -> Int] = Vec[fn() -> Int].new();
  fs.push(ten);
  if fs[0]() != 10 { return 1; }
  if apply(fs[0]) != 10 { return 2; }

  var s = Suite{ tests: [ten] };
  if s.tests[0]() != 10 { return 3; }

  var closures = [fn(x: Int) -> Int { return x + 1; }];
  if closures[0](2) != 3 { return 4; }

  var g = fs[0];
  if g() != 10 { return 5; }

  var fns = [ten, ten];
  var total = 0;
  for f in fns { total = total + f(); }
  if total != 20 { return 6; }

  return 0;
}
