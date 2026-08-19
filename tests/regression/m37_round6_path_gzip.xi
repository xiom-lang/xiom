// Round-6 regressions (2026-08-19):
// 1. gzip_decompress returned Ok for garbage input — the contract-ensure
//    Imply (`result is Ok => result.len() >= 0`) compiled the consequence
//    UNCONDITIONALLY: for an Err result the payload unbox inttoptr'd 0 and
//    loaded from NULL (UB) → the Err return value got corrupted → Ok path.
//    The Imply consequence is now short-circuited (guarded block, default
//    true on the false path).
// 2. `let name = f()?;` bound the Option[Str] payload as an i64 slot; the
//    Str builtin handler for byte_at passed the SLOT ADDRESS as the string
//    (the "&mut self" receiver heuristic fired on the i8* param) → garbage
//    → io.extension/path.extension returned None. The Try binding now
//    records the payload XIOM type and the receiver heuristic skips Str.
// 3. `s.substr(...)` had NO inline handler → call to a never-defined
//    @Str.substr → zero-param stub `ret i64 0` → NULL string → 0xC0000005
//    in every parent_path/file_name user. substr now lowers to
//    xiom_str_slice like slice.
// 4. Str builtin handlers (slice/substr/starts_with/ends_with) hijacked
//    non-Str receivers: `p.starts_with(b)` on a Path struct BOXED the Path
//    and passed the box address as a string → always false. The handlers
//    now verify the receiver is really a Str (receiver_is_str).
// 5. path.xi (catalog) called bare `join_paths` with no import — the bare
//    call resolved to no registered key → zero-param stub → NULL → crash.
//    path.xi now imports xiom.env (join_paths + FAMILY-aware separator).
module m37_round6_path_gzip
use xiom.compress;
use xiom.path;
use xiom.io;

fn main() -> Int {
  // 1: garbage gzip input must Err, not Ok.
  var small = Vec[UInt8].new();
  small.push(0);
  small.push(1);
  small.push(2);
  var r = compress.gzip_decompress(&small);
  var ok = false;
  match r {
    Ok(_) => { ok = true; }
    Err(_) => {}
  }
  if ok { return 1; }

  // 5: join chain (was stub-NULL crash before the env import).
  var p = path.Path.new("/home/user");
  var j = p.join("a").join("b").join("c");
  var js = j.as_path().to_str();
  var sep = path.path_separator();
  if js != "/home/user" + sep + "a" + sep + "b" + sep + "c" { return 2; }

  // 2/3/4: file_name / extension via the ?-bound Str payloads.
  match p.file_name() {
    Some(n) => {
      if n != "user" { return 3; }
    }
    None => { return 4; }
  }
  match io.extension("file.xi") {
    Some(e) => {
      if e != "xi" { return 5; }
    }
    None => { return 6; }
  }
  match io.parent_path("/usr/bin/xiom") {
    Some(par) => {
      if par != "/usr/bin" { return 7; }
    }
    None => { return 8; }
  }
  // 4: Path.starts_with/ends_with must NOT be hijacked by the Str builtin.
  var p2 = path.Path.new("/usr/local/bin/xiom");
  if !p2.starts_with(path.Path.new("/usr")) { return 9; }
  if !p2.ends_with(path.Path.new("xiom")) { return 10; }
  return 0;
}
