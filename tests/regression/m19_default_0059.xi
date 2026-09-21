// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0059

interface Status {
  fn status(&self) -> Str {
    match code() {
      200 => "OK",
      404 => "NotFound",
      _ => "Unknown"
    }
  }
  fn code(&self) -> Int;
}

type Response = { status_code: Int; }

fn Response.code(&self) -> Int { return status_code; }

fn main() -> Int {
  var r200: Response = Response{ status_code: 200 };
  var r404: Response = Response{ status_code: 404 };
  var r500: Response = Response{ status_code: 500 };
  if r200.status() == "OK" && r404.status() == "NotFound" && r500.status() == "Unknown" { return 0; }
  return 1;
}
