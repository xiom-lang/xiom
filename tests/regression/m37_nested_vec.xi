module m37_nested_vec
// BUG 23 #2 regression: nested Vec[Vec[T]] element reads.
// The parser converted Vec[Vec[Int]] generic args to Ident("_"), the
// element size fell to 8 bytes (truncating every inner Vec to its data
// pointer), and m[i][j] reads returned garbage/0.

fn main() -> Int {
  var m = Vec[Vec[Int]].new();
  var r1 = Vec[Int].new();
  r1.push(1); r1.push(2);
  var r2 = Vec[Int].new();
  r2.push(3); r2.push(4);
  m.push(r1); m.push(r2);
  var x = m[0][1];
  if x != 2 { return 1; }
  var y = m[1][0];
  if y != 3 { return 2; }
  var row = m[1];
  if row.len() != 2 { return 3; }
  if m.len() != 2 { return 4; }
  // float inner Vecs
  var fm = Vec[Vec[Float64]].new();
  var fr = Vec[Float64].new();
  fr.push(1.5); fr.push(2.5);
  fm.push(fr);
  if fm[0][1] != 2.5 { return 5; }
  return 0;
}
