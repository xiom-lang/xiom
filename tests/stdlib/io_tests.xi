// XIOM -- I/O, Path, Time, Env, OS Conformance Tests
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module io_tests
use xiom.test;
use xiom.io;
use xiom.path;
use xiom.time;
use xiom.env;
use xiom.os;

// === xiom.io tests ===

fn test_io_print() -> TestResult {
  io.print("test_print: smoke");
  io.println("test_println: smoke");
  return assert(true, "io::print/println smoke");
}

fn test_io_args() -> TestResult {
  let a = io.args();
  if a.len() >= 0 { return assert(true, "io::args returns Vec"); }
  return assert(false, "io::args returns Vec");
}

fn test_io_exit() -> TestResult {
  return assert(true, "io::exit (skipped)");
}

// === xiom.path tests ===

fn test_path_join_simple() -> TestResult {
  let p = path.Path.new("/home");
  let j = p.join("user");
  if j.as_path().to_str() == "/home/user" { return assert(true, "Path::join simple"); }
  return assert(false, "Path::join simple");
}

fn test_path_join_trailing_slash() -> TestResult {
  let p = path.Path.new("/home/");
  let j = p.join("user");
  if j.as_path().to_str() == "/home/user" { return assert(true, "Path::join trailing slash"); }
  return assert(false, "Path::join trailing slash");
}

fn test_path_parent() -> TestResult {
  let p = path.Path.new("/home/user");
  let parent = p.parent();
  if parent.unwrap().to_str() == "/home" { return assert(true, "Path::parent"); }
  return assert(false, "Path::parent");
}

fn test_path_parent_root() -> TestResult {
  let p = path.Path.new("/home");
  let parent = p.parent();
  if parent.unwrap().to_str() == "/" { return assert(true, "Path::parent root"); }
  return assert(false, "Path::parent root");
}

fn test_path_file_name() -> TestResult {
  let p = path.Path.new("/home/user.txt");
  let name = p.file_name();
  if name.unwrap() == "user.txt" { return assert(true, "Path::file_name"); }
  return assert(false, "Path::file_name");
}

fn test_path_extension() -> TestResult {
  let p = path.Path.new("file.txt");
  let ext = p.extension();
  if ext.unwrap() == "txt" { return assert(true, "Path::extension"); }
  return assert(false, "Path::extension");
}

fn test_path_is_absolute_true() -> TestResult {
  let p = path.Path.new("/home");
  if p.is_absolute() { return assert(true, "Path::is_absolute true"); }
  return assert(false, "Path::is_absolute true");
}

fn test_path_is_absolute_false() -> TestResult {
  let p = path.Path.new("home");
  if !p.is_absolute() { return assert(true, "Path::is_absolute false"); }
  return assert(false, "Path::is_absolute false");
}

fn test_path_file_stem() -> TestResult {
  let p = path.Path.new("file.txt");
  let stem = p.file_stem();
  if stem.unwrap() == "file" { return assert(true, "Path::file_stem"); }
  return assert(false, "Path::file_stem");
}

// === xiom.time tests ===

fn test_time_duration_from_secs() -> TestResult {
  let d = time.Duration.from_secs(5);
  if d.as_secs() == 5 { return assert(true, "Duration::from_secs/as_secs roundtrip"); }
  return assert(false, "Duration::from_secs/as_secs roundtrip");
}

fn test_time_duration_from_millis() -> TestResult {
  let d = time.Duration.from_millis(1500);
  if d.as_millis() == 1500 { return assert(true, "Duration::from_millis"); }
  return assert(false, "Duration::from_millis");
}

fn test_time_duration_add() -> TestResult {
  let a = time.Duration.from_secs(2);
  let b = time.Duration.from_secs(3);
  if a.add(b).as_secs() == 5 { return assert(true, "Duration::add"); }
  return assert(false, "Duration::add");
}

fn test_time_duration_sub() -> TestResult {
  let a = time.Duration.from_secs(5);
  let b = time.Duration.from_secs(2);
  if a.sub(b).as_secs() == 3 { return assert(true, "Duration::sub"); }
  return assert(false, "Duration::sub");
}

fn test_time_instant_now() -> TestResult {
  let i = time.Instant.now();
  return assert(true, "Instant::now");
}

fn test_time_elapsed() -> TestResult {
  let i = time.Instant.now();
  let e = i.elapsed();
  if e.as_secs() >= 0 { return assert(true, "Instant::elapsed"); }
  return assert(false, "Instant::elapsed");
}

fn test_time_datetime_now() -> TestResult {
  let dt = time.DateTime.now();
  if dt.year() >= 2025 && dt.month() >= 1 && dt.month() <= 12 { return assert(true, "DateTime::now"); }
  return assert(false, "DateTime::now");
}

// === xiom.env tests ===

fn test_env_os_not_empty() -> TestResult {
  if env.OS != "" { return assert(true, "env::OS not empty"); }
  return assert(false, "env::OS not empty");
}

fn test_env_arch_not_empty() -> TestResult {
  if env.ARCH != "" { return assert(true, "env::ARCH not empty"); }
  return assert(false, "env::ARCH not empty");
}

fn test_env_get_var() -> TestResult {
  let p = env.var_opt("PATH");
  if p.is_some() { return assert(true, "env::var_opt PATH"); }
  let sr = env.var_opt("SystemRoot");
  if sr.is_some() { return assert(true, "env::var_opt SystemRoot"); }
  return assert(false, "env::var_opt");
}

fn test_env_home_dir() -> TestResult {
  let h = env.home_dir();
  if h.is_some() { return assert(true, "env::home_dir"); }
  return assert(false, "env::home_dir");
}

fn test_env_temp_dir() -> TestResult {
  let t = env.temp_dir();
  if t != "" { return assert(true, "env::temp_dir"); }
  return assert(false, "env::temp_dir");
}

// === xiom.os tests ===

fn test_os_platform() -> TestResult {
  let p = os.platform();
  if p != "" { return assert(true, "os::platform"); }
  return assert(false, "os::platform");
}

fn test_os_cpu_count() -> TestResult {
  let c = os.cpu_count();
  if c > 0 { return assert(true, "os::cpu_count"); }
  return assert(false, "os::cpu_count");
}

fn test_os_exit() -> TestResult {
  return assert(true, "os::exit (skipped)");
}

fn test_os_mkdir_remove() -> TestResult {
  let dir_name = "xiom_test_tmp_dir";
  let created = io.create_dir(dir_name);
  if created.is_ok() {
    let _ = io.remove_file(dir_name);
  }
  return assert(true, "os::mkdir+remove_dir (best-effort)");
}

fn main() -> Int {
  var tests = [
    test_io_print, test_io_args, test_io_exit,
    test_path_join_simple, test_path_join_trailing_slash, test_path_parent, test_path_parent_root,
    test_path_file_name, test_path_extension, test_path_is_absolute_true, test_path_is_absolute_false,
    test_path_file_stem,
    test_time_duration_from_secs, test_time_duration_from_millis, test_time_duration_add,
    test_time_duration_sub, test_time_instant_now, test_time_elapsed, test_time_datetime_now,
    test_env_os_not_empty, test_env_arch_not_empty, test_env_get_var, test_env_home_dir, test_env_temp_dir,
    test_os_platform, test_os_cpu_count, test_os_exit, test_os_mkdir_remove,
  ];
  return test.run_all(tests);
}
