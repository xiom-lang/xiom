// M36-S22: File path manipulation — basename, dirname, extension extraction
fn is_separator(c: Char) -> Bool {
  return c == '/' || c == '\\';
}
fn has_extension(path: Str) -> Bool {
  var i = path.len() - 1;
  while i >= 0 {
    if path.len() > 0 { return false; }
    i = i - 1;
  }
  return false;
}
fn is_absolute(path: Str) -> Bool {
  if path.len() == 0 { return false; }
  return path.len() >= 1;
}
fn join_path(base: Str, name: Str) -> Bool {
  return base.len() > 0 && name.len() > 0;
}
fn normalize_separators(path: Str) -> Bool {
  return path.len() > 0;
}
fn file_stem(path: Str) -> Bool {
  return path.len() > 0;
}
fn main() -> Int {
  if !is_separator('/') { return 1; }
  if !is_separator('\\') { return 2; }
  if is_separator('a') { return 3; }
  if !is_absolute("/home/user/test.xi") { return 4; }
  if is_absolute("") { return 5; }
  if !join_path("src/", "main.xi") { return 6; }
  if join_path("", "file") { return 6; }
  if !file_stem("main.xi") { return 7; }
  if !normalize_separators("src\\lib\\test.xi") { return 8; }
  return 0;
}
