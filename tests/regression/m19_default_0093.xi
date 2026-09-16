// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0093

interface Labeled {
  fn label(&self) -> Str { return to_str(); }
  fn to_str(&self) -> Str;
}

enum LogLevel {
  Debug,
  Info,
  Warn,
  Error
}

fn LogLevel.to_str(&self) -> Str {
  match self {
    Debug => "DEBUG",
    Info => "INFO",
    Warn => "WARN",
    Error => "ERROR"
  }
}

fn main() -> Int {
  var d: LogLevel = LogLevel.Debug;
  var i: LogLevel = LogLevel.Info;
  var w: LogLevel = LogLevel.Warn;
  var e: LogLevel = LogLevel.Error;
  if d.to_str() != "DEBUG" { return 1; }
  if d.label() != "DEBUG" { return 2; }
  if i.to_str() != "INFO" { return 3; }
  if i.label() != "INFO" { return 4; }
  if w.to_str() != "WARN" { return 5; }
  if w.label() != "WARN" { return 6; }
  if e.to_str() != "ERROR" { return 7; }
  if e.label() != "ERROR" { return 8; }
  return 0;
}
