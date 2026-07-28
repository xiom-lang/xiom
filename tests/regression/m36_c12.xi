// M36-C12: Every loop pattern — while true, while cond, while break, while continue, nested loops, infinite+break
fn test_while_true() -> Int {
  var i = 0;
  var acc = 0;
  while true {
    acc = acc + i;
    i = i + 1;
    if i >= 10 { break; }
  }
  if acc != 45 { return 1; }
  return 0;
}
fn test_while_cond() -> Int {
  var i = 0; var s = 0;
  while i < 5 { s = s + i; i = i + 1; }
  if s != 10 { return 1; }
  return 0;
}
fn test_while_break() -> Int {
  var i = 0; var found = 0;
  while i < 100 {
    if i == 42 { found = i; break; }
    i = i + 1;
  }
  if found != 42 { return 1; }
  return 0;
}
fn test_while_continue() -> Int {
  var i = 0; var s = 0;
  while i < 10 {
    i = i + 1;
    if i % 2 == 0 { continue; }
    s = s + i;
  }
  if s != 25 { return 1; }
  return 0;
}
fn test_nested_loops() -> Int {
  var i = 0; var s = 0;
  while i < 5 {
    var j = 0;
    while j < 5 {
      s = s + 1;
      j = j + 1;
    }
    i = i + 1;
  }
  if s != 25 { return 1; }
  return 0;
}
fn test_infinite_break() -> Int {
  var i = 0; var limit = 0;
  while true {
    limit = limit + 1;
    if limit > 50 { break; }
    i = i + limit;
  }
  if i != 1275 { return 1; }
  return 0;
}
fn test_while_break_nested() -> Int {
  var outer = 0; var total = 0;
  while outer < 5 {
    var inner = 0;
    while inner < 10 {
      total = total + 1;
      inner = inner + 1;
      if inner >= 3 { break; }
    }
    outer = outer + 1;
  }
  if total != 15 { return 1; }
  return 0;
}
fn main() -> Int {
  if test_while_true() != 0 { return 1; }
  if test_while_cond() != 0 { return 2; }
  if test_while_break() != 0 { return 3; }
  if test_while_continue() != 0 { return 4; }
  if test_nested_loops() != 0 { return 5; }
  if test_infinite_break() != 0 { return 6; }
  if test_while_break_nested() != 0 { return 7; }
  return 0;
}
