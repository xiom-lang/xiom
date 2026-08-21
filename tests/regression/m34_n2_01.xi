// M34-N2-01: 10-deep if-else chain -- pushes conditional nesting limit
fn main() -> Int {
  var x = 1;
  if x == 1 { if x == 1 { if x == 1 { if x == 1 { if x == 1 {
    if x == 1 { if x == 1 { if x == 1 { if x == 1 { if x == 1 {
      var r = 0;
      return r;
    } else { return 1; }
    } else { return 1; }
    } else { return 1; }
    } else { return 1; }
    } else { return 1; }
  } else { return 1; } } else { return 1; } } else { return 1; } } else { return 1; } } else { return 1; }
  return 1;
}
