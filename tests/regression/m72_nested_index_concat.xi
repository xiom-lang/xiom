module m72_nested_index_concat
// R17 regression: R14's concat-operand LLVM fallback misclassified
// NESTED-index Str elements (`rows[0][0]` of a Vec[Vec[Str]]) as integers
// and printed the pointer value (smoke_serialize_csv). `expr_is_integer`
// now resolves nested element types recursively; known Str elements take
// inttoptr (pointer recovery), unknown chained-collect Vec[Int] elements
// still use the LLVM scalar fallback (R14 lock: m71).

use xiom.io;

fn main() -> Int {
  var r = Vec[Str].new();
  r.push("a"); r.push("b");
  var rows = Vec[Vec[Str]].new();
  rows.push(r);

  // Single-index Vec[Str] element in concat.
  var s1 = "";
  s1 = s1 + r[0];
  io.println("single=[" + s1 + "]");

  // Nested-index element in concat (the CSV flat() shape).
  var s2 = "";
  s2 = s2 + rows[0][0];
  io.println("nested-assign=[" + s2 + "]");

  // Nested-index element inline in a concat expression.
  var s3 = "";
  s3 = s3 + rows[0][1];
  io.println("nested-inline=[" + s3 + "]");

  if s1 != "a" { io.println("single mismatch"); return 1; }
  if s2 != "a" { io.println("nested mismatch"); return 2; }
  if s3 != "b" { io.println("nested inline mismatch"); return 3; }
  return 0;
}
