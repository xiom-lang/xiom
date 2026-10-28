module result_bool_patterns
use xiom.io;
fn t() -> Result[Bool, Str] { return Ok(true); }
fn f() -> Result[Bool, Str] { return Ok(false); }
fn e() -> Result[Bool, Str] { return Err("x"); }
fn s() -> Option[Bool] { return Some(true); }
fn main() -> Int {
  match t() {
    Err(_) => { return 1; },
    Ok(false) => { return 2; },
    Ok(true) => { },
  }
  match f() {
    Err(_) => { return 3; },
    Ok(false) => { },
    Ok(true) => { return 4; },
  }
  match e() {
    Ok(false) => { return 5; },
    Ok(true) => { return 6; },
    Err(_) => { },
  }
  match s() {
    Some(false) => { return 7; },
    Some(true) => { },
    None => { return 8; },
  }
  // or-pattern with Bool literals
  match t() {
    Ok(true) | Ok(false) => { },
    Err(_) => { return 9; },
  }
  io.println("bool payload patterns ok");
  return 0;
}
