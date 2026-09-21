// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0039

interface Stats {
  fn stats(&self) -> Int { return min() + max() + avg() + sum(); }
  fn min(&self) -> Int;
  fn max(&self) -> Int;
  fn avg(&self) -> Int;
  fn sum(&self) -> Int;
}

type Dataset = { lo: Int; hi: Int; mid: Int; total: Int; }

fn Dataset.stats(self) -> Int { return self.min() + self.max() + self.avg() + self.sum(); }


fn Dataset.min(&self) -> Int { return lo; }

fn Dataset.max(&self) -> Int { return hi; }

fn Dataset.avg(&self) -> Int { return mid; }

fn Dataset.sum(&self) -> Int { return total; }

fn main() -> Int {
  var d: Dataset = Dataset{ lo: 1, hi: 10, mid: 5, total: 55 };
  if d.stats() == 71 { return 0; }
  return 1;
}
