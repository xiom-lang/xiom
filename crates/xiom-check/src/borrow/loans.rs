// XIOM -- Loan Tracking with Place Model (5c-R: field-granular borrows)
// Extends the existing lexical BorrowChecker with Place-level conflict detection.

use super::place::{Place, PlaceConflict, places_conflict};

/// A single loan: a borrow of a specific place with a read/write kind.
#[derive(Debug, Clone)]
pub struct Loan {
    pub place: Place,
    pub is_write: bool,
}

impl Loan {
    pub fn read(place: Place) -> Self { Self { place, is_write: false } }
    pub fn write(place: Place) -> Self { Self { place, is_write: true } }
}

/// The active set of loans at a point in the program.
#[derive(Debug, Clone, Default)]
pub struct LoanSet {
    loans: Vec<Loan>,
}

impl LoanSet {
    pub fn new() -> Self { Self { loans: Vec::new() } }

    /// Register a new loan. Returns `true` if the loan was accepted.
    pub fn grant(&mut self, loan: Loan) -> LoanResult {
        // Check conflicts against all existing active loans
        for existing in &self.loans {
            match places_conflict(&existing.place, &loan.place) {
                PlaceConflict::Disjoint => continue, // no conflict
                PlaceConflict::Overlap | PlaceConflict::Ambiguous => {
                    // Write-write conflict
                    if existing.is_write && loan.is_write {
                        return LoanResult::Conflict(format!(
                            "cannot borrow `{}` as mutable because it is already borrowed as mutable",
                            self.place_display(&loan.place)
                        ));
                    }
                    // Read-write conflict (write after read, or read after write)
                    if existing.is_write || loan.is_write {
                        return LoanResult::Conflict(format!(
                            "cannot borrow `{}` as {} because it is also borrowed as {}",
                            self.place_display(&loan.place),
                            if loan.is_write { "mutable" } else { "immutable" },
                            if existing.is_write { "mutable" } else { "immutable" }
                        ));
                    }
                }
            }
        }
        self.loans.push(loan);
        LoanResult::Granted
    }

    /// Release all loans for a given variable (e.g., when it goes out of scope).
    pub fn release_var(&mut self, local: &str) {
        self.loans.retain(|l| l.place.local != local);
    }

    /// Release all active loans (at end of scope).
    pub fn release_all(&mut self) {
        self.loans.clear();
    }

    /// Human-readable display of a place.
    fn place_display(&self, p: &Place) -> String {
        let mut s = p.local.clone();
        for proj in &p.projs {
            match proj {
                super::place::Projection::Field(name) => { s.push('.'); s.push_str(name); }
                super::place::Projection::Index => s.push_str("[_]"),
                super::place::Projection::Deref => s.push_str(".*"),
                super::place::Projection::Cast => s.push_str(" as _"),
                super::place::Projection::Subslice => s.push_str("[..]"),
            }
        }
        s
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoanResult {
    Granted,
    Conflict(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loan_read_read_no_conflict() {
        let mut ls = LoanSet::new();
        let p = Place::from_local("x");
        assert_eq!(ls.grant(Loan::read(p.clone())), LoanResult::Granted);
        assert_eq!(ls.grant(Loan::read(p.clone())), LoanResult::Granted);
    }

    #[test]
    fn test_loan_write_write_conflict() {
        let mut ls = LoanSet::new();
        let p = Place::from_local("x");
        assert_eq!(ls.grant(Loan::write(p.clone())), LoanResult::Granted);
        assert!(matches!(ls.grant(Loan::write(p.clone())), LoanResult::Conflict(_)));
    }

    #[test]
    fn test_loan_read_write_conflict() {
        let mut ls = LoanSet::new();
        let p = Place::from_local("x");
        assert_eq!(ls.grant(Loan::read(p.clone())), LoanResult::Granted);
        assert!(matches!(ls.grant(Loan::write(p.clone())), LoanResult::Conflict(_)));
    }

    #[test]
    fn test_loan_different_fields_no_conflict() {
        let mut ls = LoanSet::new();
        let a = Place::from_local("p").field("x");
        let b = Place::from_local("p").field("y");
        // Read a.x, write a.y -- disjoint fields, no conflict
        assert_eq!(ls.grant(Loan::read(a)), LoanResult::Granted);
        assert_eq!(ls.grant(Loan::write(b)), LoanResult::Granted);
    }

    #[test]
    fn test_loan_prefix_conflict() {
        let mut ls = LoanSet::new();
        let a = Place::from_local("a").field("b");
        let b = Place::from_local("a").field("b").field("c");
        // Write a.b, then read a.b.c -- prefix => conflict
        assert_eq!(ls.grant(Loan::write(a)), LoanResult::Granted);
        assert!(matches!(ls.grant(Loan::read(b)), LoanResult::Conflict(_)));
    }
}
