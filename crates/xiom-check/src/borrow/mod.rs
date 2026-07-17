// XIOM — Borrow Checker (5c-R: field-granular borrows, WS2 #1)
// Phase 1: Place model, places_conflict walk, loan tracking

pub mod place;

pub use place::{Place, Projection, PlaceConflict, places_conflict};
