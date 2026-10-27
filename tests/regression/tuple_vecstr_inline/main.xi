// REL-20261023-1854-pulse lock: a tuple-returned Vec[Str] indexed INLINE
// (`pq.0[0]` inside a concatenation or as a call argument) must resolve the
// Str element, not a pointer/Int. v0.64.8 printed pointer values unless the
// element was first bound (`let n0: Str = pq.0[0]`).
module tuple_vecstr_inline

use xiom.io;

fn parse_query(s: Str) -> (Vec[Str], Vec[Str]) {
  var names = Vec[Str].new();
  names.push("token");
  names.push("x");
  var values = Vec[Str].new();
  values.push("secret");
  values.push("1");
  return (names, values);
}

fn is_token(s: Str) -> Bool {
  return s == "token";
}

fn main() -> Int {
  let pq = parse_query("token=secret&x=1");
  // Inline concatenation: element type must be Str.
  if "k=" + pq.0[0] != "k=token" { return 1; }
  if "v=" + pq.1[0] != "v=secret" { return 2; }
  // Inline call argument: must pass the Str value.
  if !is_token(pq.0[0]) { return 3; }
  if is_token(pq.0[1]) { return 4; }
  // Annotated form keeps working.
  let n0: Str = pq.0[0];
  if n0 != "token" { return 5; }
  io.println("tuple inline vec-str ok n0=" + pq.0[0]);
  return 0;
}
