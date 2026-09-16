// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM -- Borrow Checker (5c-R: field-granular borrows, WS2 #1)
// Phase 1: Place model, places_conflict walk, loan tracking

pub mod place;
pub mod loans;

pub use place::{Place, Projection, PlaceConflict, places_conflict};
pub use loans::{Loan, LoanSet, LoanResult};
