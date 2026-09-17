// R39 lock: m84.beta.Metrics (4 fields) -- same leaf as m84.alpha.Metrics.
// Pre-R39 the catalog injection flattened both to a bare `%struct.Metrics`
// (first module won), so beta's literal GEPed field 3 of a 2-field definition
// and the program either failed clang or read the wrong fields.
module m84.beta

pub type Metrics = {
  w: Int;
  x: Int;
  y: Int;
  z: Int;
}

pub fn make() -> Metrics {
  return Metrics{ w: 1, x: 2, y: 3, z: 4 };
}

pub fn total(m: &Metrics) -> Int {
  return m.w + m.x + m.y + m.z;
}
