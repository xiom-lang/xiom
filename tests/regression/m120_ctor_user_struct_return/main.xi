// m120 (R67): the Some/None/Ok/Err constructors chose their container struct
// with a SUBSTRING test on the enclosing function's return type
// (`ctor_ret.contains("Option"/"Result")`). A function whose return type was a
// USER struct containing that substring -- `TestResult`, `Options` -- made the
// constructor build the wrong struct: `Ok(5)` inside `fn f() -> TestResult`
// emitted a `%struct.TestResult` payload and clang rejected the IR
// ("invalid getelementptr indices"); a struct with a matching field count could
// silently corrupt. The check is now the strict container-leaf test
// (leaf == "Result"/"Option" or a concrete `Result__A__B`/`Option__T`).
module m120_ctor_user_struct_return

type TestResult = { passed: Bool; name: Str; }
type Options = { val: Int; }
type MyOption = { a: Int; b: Int; }
type Point = { x: Int; y: Int; }

fn helper(flag: Bool) -> TestResult {
  var okv = Ok(5);
  var errv = Err("boom");
  match okv {
    Ok(v) => { if v != 5 { return TestResult{ passed: false; name: "okv" }; } },
    Err(_) => { return TestResult{ passed: false; name: "okv-e" }; },
  };
  match errv {
    Ok(_) => { return TestResult{ passed: false; name: "errv-o" }; },
    Err(e) => { if e != "boom" { return TestResult{ passed: false; name: "errv" }; } },
  };
  if flag { return TestResult{ passed: true; name: "ok" }; }
  return TestResult{ passed: false; name: "no" };
}

fn make_options() -> Options {
  var o = Some(3);
  var n = None;
  match o {
    Some(v) => { if v != 3 { return Options{ val: -1 }; } },
    None => { return Options{ val: -2 }; },
  };
  match n {
    Some(_) => { return Options{ val: -3 }; },
    None => {},
  };
  return Options{ val: 1 };
}

fn make_myoption() -> MyOption {
  var o = Some(9);
  match o {
    Some(v) => { if v == 9 { return MyOption{ a: 1; b: 2 }; } },
    None => {},
  };
  return MyOption{ a: 0; b: 0 };
}

// Concrete container returns must still build the CONCRETE struct (the
// 5c.35 behavior the substring test was protecting).
fn make_opt() -> Option[Point] {
  var p = Point{ x: 1; y: 2 };
  return Some(p);
}

fn make_res() -> Result[Point, Str] {
  var p = Point{ x: 5; y: 6 };
  return Ok(p);
}

fn main() -> Int {
  var tr = helper(true);
  if !tr.passed { return 1; }
  if tr.name != "ok" { return 2; }

  var op = make_options();
  if op.val != 1 { return 3; }

  var mo = make_myoption();
  if mo.a != 1 || mo.b != 2 { return 4; }

  match make_opt() {
    Some(p) => { if p.x != 1 || p.y != 2 { return 5; } },
    None => { return 6; },
  };
  match make_res() {
    Ok(p) => { if p.x != 5 || p.y != 6 { return 7; } },
    Err(_) => { return 8; },
  };
  return 0;
}
