module m100_result_payload_contract

// R49-3 lock (stdlib relay p_result_payload_contract): one module with a
// scalar-payload Result contract PLUS a Vec-payload Result contract. The
// `.value.len()` dispatch must see the rebound payload container (was
// xiom_str_len on a %struct.Vec -> clang type error). The Err form reads
// `result.value` after `is Err`.

fn scalar_payload(n: Int) -> Result[Int, Str]
  ensures: result is Ok => result.value >= 0
{
  if n < 0 { return Err("neg"); }
  return Ok(n);
}

fn vec_payload() -> Result[Vec[UInt8], Str]
  ensures: result is Ok => result.value.len() >= 0
{
  return Ok(Vec[UInt8].new());
}

fn err_payload(flag: Bool) -> Result[Int, Vec[UInt8]]
  ensures: result is Err => result.value.len() >= 0
{
  if flag { return Err(Vec[UInt8].new()); }
  return Ok(1);
}

fn main() -> Int {
  let a = scalar_payload(5);
  let b = vec_payload();
  let c = err_payload(true);
  return 0;
}