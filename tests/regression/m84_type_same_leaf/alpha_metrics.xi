// R39 lock: m84.alpha.Metrics (2 fields) -- same leaf as m84.beta.Metrics.
module m84.alpha

pub type Metrics = {
  a: Int;
  b: Int;
}

pub fn make() -> Metrics {
  return Metrics{ a: 3, b: 4 };
}

pub fn total(m: &Metrics) -> Int {
  return m.a + m.b;
}
