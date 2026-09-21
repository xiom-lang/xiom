// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0023

interface AgeCheck {
  fn is_teen(&self) -> Bool { return age() >= 13 && age() <= 19; }
  fn age(&self) -> Int;
}

type Profile = { years: Int; }

fn Profile.is_teen(self) -> Bool { return self.age() >= 13 && self.age() <= 19; }


fn Profile.age(&self) -> Int { return years; }

fn main() -> Int {
  var teen: Profile = Profile{ years: 15 };
  var adult: Profile = Profile{ years: 25 };
  var child: Profile = Profile{ years: 10 };
  if teen.is_teen() && !adult.is_teen() && !child.is_teen() { return 0; }
  return 1;
}
