// m278 (REL-20261011-2137-engine) collision partner: same bare type name as
// coll.zuser's Mesh, different field ORDER (vertices at index 1). The
// catalog-sorted injection order puts this module first, so pre-fix its
// layout claimed the bare key and coll.zuser's own code compiled against it.
module coll.aother;

pub type Mesh = { tag: Int; vertices: Vec[Int]; }

pub fn marker() -> Int { return 1; }
