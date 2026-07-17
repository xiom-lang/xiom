// XIOM — Place Model (5c-R: field-granular borrows, WS2 #1)
// Direct transplant from rustc: `compiler/rustc_middle/src/mir/syntax.rs` (~line 1162)
// and `compiler/rustc_borrowck/src/places_conflict.rs`.

// ============================================================================
// Projection — one step in a place path
// ============================================================================

/// A single access step along a borrow path. `a.b[i].c` is:
///   root local "a" + [Field("b"), Index, Field("c")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Projection {
    /// `expr.field` — named struct field access
    Field(String),
    /// `expr[idx]` — runtime index (Array/Vec/Map access)
    Index,
    /// `*expr` — pointer dereference
    Deref,
    /// `expr as Type` — type cast
    Cast,
    /// Sub-slice `expr[start..end]`
    Subslice,
}

// ============================================================================
// Place — a rooted path
// ============================================================================

/// A place is a root local variable with zero or more projections.
/// `place.local = "a"` + `place.projs = [Field("x"), Index, Field("y")]`
/// represents `a.x[idx].y`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Place {
    /// Name of the root local variable (or `"return"` for the return slot).
    pub local: String,
    /// Projection chain from root to the accessed field/element.
    pub projs: Vec<Projection>,
}

impl Place {
    /// Create a place for a bare local variable.
    pub fn from_local(name: &str) -> Self {
        Self { local: name.to_string(), projs: vec![] }
    }

    /// Push a field access projection onto the place.
    pub fn field(mut self, name: &str) -> Self {
        self.projs.push(Projection::Field(name.to_string()));
        self
    }

    /// Push an index projection onto the place.
    pub fn index(mut self) -> Self {
        self.projs.push(Projection::Index);
        self
    }

    /// Push a deref projection onto the place.
    pub fn deref(mut self) -> Self {
        self.projs.push(Projection::Deref);
        self
    }

    /// True when this place is a prefix of `other` — e.g., `a.b` is a prefix
    /// of `a.b.c`. The prefix rule: if one place is a prefix of another, they
    /// conflict.
    pub fn is_prefix_of(&self, other: &Place) -> bool {
        if self.local != other.local { return false; }
        if self.projs.len() > other.projs.len() { return false; }
        self.projs.iter().zip(other.projs.iter()).all(|(a, b)| a == b)
    }
}

// ============================================================================
// places_conflict — the lockstep walk (directly portable from rustc)
// ============================================================================

/// Result of the place-conflict walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceConflict {
    /// The two places are provably disjoint (different struct fields,
    /// different enum variants, etc.) — no conflict.
    Disjoint,
    /// The two places definitely overlap — conflict.
    Overlap,
    /// At least one Index projection prevented resolving the overlap;
    /// the caller must conservatively assume conflict.
    Ambiguous,
}

/// Determine whether two places conflict. This is a lockstep walk:
/// walk both projection lists element by element until either:
///   - They diverge at a Field(`name_a`) vs Field(`name_b`) with name_a != name_b
///     → DISJOINT (different fields of the same struct are non-overlapping)
///   - One list is exhausted while the other still has elements
///     → the shorter is a prefix of the longer → OVERLAP
///   - Both lists exhaust simultaneously → OVERLAP (same place)
///   - An Index projection is encountered → AMBIGUOUS (runtime index)
///   - A Deref projection is encountered → OVERLAP (conservative)
///
/// Direct transplant from `compiler/rustc_borrowck/src/places_conflict.rs`.
pub fn places_conflict(a: &Place, b: &Place) -> PlaceConflict {
    // Different root locals — no conflict (stack slots don't alias).
    if a.local != b.local {
        return PlaceConflict::Disjoint;
    }

    let a_projs = &a.projs;
    let b_projs = &b.projs;
    let min_len = std::cmp::min(a_projs.len(), b_projs.len());

    for i in 0..min_len {
        match (&a_projs[i], &b_projs[i]) {
            // Same element — continue walking
            (Projection::Field(a_name), Projection::Field(b_name))
                if a_name == b_name => continue,
            // Different named fields — provably disjoint
            (Projection::Field(_), Projection::Field(_)) => return PlaceConflict::Disjoint,
            // Index: runtime value, can't prove disjointness
            (Projection::Index, _) | (_, Projection::Index) => return PlaceConflict::Ambiguous,
            // Deref: always conservatively assume overlap
            (Projection::Deref, _) | (_, Projection::Deref) => return PlaceConflict::Overlap,
            // Different projection kinds (e.g., Field vs Index) — ambiguous
            _ => return PlaceConflict::Ambiguous,
        }
    }

    // Walk exhausted for both — same place (overlap)
    // or one is a prefix of the other (also overlap)
    PlaceConflict::Overlap
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_places_conflict_same_root_no_projs() {
        let a = Place::from_local("x");
        let b = Place::from_local("x");
        assert_eq!(places_conflict(&a, &b), PlaceConflict::Overlap);
    }

    #[test]
    fn test_places_conflict_different_roots() {
        let a = Place::from_local("x");
        let b = Place::from_local("y");
        assert_eq!(places_conflict(&a, &b), PlaceConflict::Disjoint);
    }

    #[test]
    fn test_places_conflict_different_fields() {
        let a = Place::from_local("p").field("x");
        let b = Place::from_local("p").field("y");
        assert_eq!(places_conflict(&a, &b), PlaceConflict::Disjoint);
    }

    #[test]
    fn test_places_conflict_same_field() {
        let a = Place::from_local("p").field("x");
        let b = Place::from_local("p").field("x");
        assert_eq!(places_conflict(&a, &b), PlaceConflict::Overlap);
    }

    #[test]
    fn test_places_conflict_prefix_rule() {
        // &a.b vs use of a.b.c — prefix → conflict
        let a = Place::from_local("a").field("b");
        let b = Place::from_local("a").field("b").field("c");
        assert_eq!(places_conflict(&a, &b), PlaceConflict::Overlap);
    }

    #[test]
    fn test_places_conflict_index_ambiguous() {
        let a = Place::from_local("v").index();
        let b = Place::from_local("v").index();
        assert_eq!(places_conflict(&a, &b), PlaceConflict::Ambiguous);
    }
}
