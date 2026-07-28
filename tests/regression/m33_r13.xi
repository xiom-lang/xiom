enum OpResult { Success(val: Int), Failure(reason: Result[Int, Str]), Unknown }
fn handle(o: OpResult) -> Int {
  match o {
    Success(v) => v,
    Failure(r) => { match r { Ok(x) => x, Err(_) => -1 } }
    Unknown => 0,
  }
}
fn main() -> Int {
  if handle(OpResult.Success(42)) != 42 { return 1; }
  if handle(OpResult.Failure(Ok(7))) != 7 { return 2; }
  if handle(OpResult.Failure(Err("bad"))) != -1 { return 3; }
  if handle(OpResult.Unknown) != 0 { return 4; }
  return 0;
}
