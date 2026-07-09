// E2E regression: Vec builtins — new/push/len/index/pop/get.
// Locks in inline Vec method dispatch + element coercion + Option payload.
// Returns 0 on success.
module e2e_vec_ops

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(10);
  v.push(20);
  v.push(30);
  if v.len() != 3 {
    return 1;
  }
  // Indexed read.
  if v[0] != 10 || v[2] != 30 {
    return 2;
  }
  // pop returns Option[Int]; Some(30).
  let last = v.pop();
  if last != Some(30) {
    return 3;
  }
  if v.len() != 2 {
    return 4;
  }
  return 0;
}
