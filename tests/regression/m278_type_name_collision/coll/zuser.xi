// m278 (REL-20261011-2137-engine): the user module. Its Mesh has vertices at
// index 0; pre-fix the bare-name winner (coll.aother, injected first) made
// make/vlen read index 1 (the Int tag) as a Vec pointer -> garbage/AV.
module coll.zuser;

pub type Mesh = { vertices: Vec[Int]; tag2: Int; }

pub fn make() -> Mesh {
  return Mesh{ vertices: Vec[Int].new(); tag2: 7 };
}

pub fn vlen(m: &Mesh) -> Int {
  return m.vertices.len();
}
