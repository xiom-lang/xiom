// M35-L13: Result size pattern -- verify Result[Int, Str] layout via match
fn main() -> Int {
  var a: Result[Int, Str] = Ok(100);
  match a {
    Ok(v) => if v != 100 { return 1; },
    Err(_) => { return 2; },
  }
  var b: Result[Int, Str] = Err("fail");
  match b {
    Ok(_) => { return 3; },
    Err(msg) => if msg == "fail" {} else { return 4; },
  }
  var c: Result[Int, Int] = Ok(77);
  match c {
    Ok(v) => if v != 77 { return 5; },
    Err(_) => { return 6; },
  }
  var d: Result[Bool, Char] = Err('E');
  match d {
    Ok(_) => { return 7; },
    Err(ch) => if ch != 'E' { return 8; },
  }
  return 0;
}
