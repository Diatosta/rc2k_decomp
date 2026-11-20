//! The oracle: the helpers as they were before their results became floats (`old`, results
//! as bits), against which the typed helpers are compared bit for bit on seeded inputs of
//! every kind (`typed`, `results`), and the helpers merged into one named function, each
//! against its old expression (`merged`).

mod merged;
#[allow(
    dead_code,
    reason = "the comparison tests call only the helpers that changed"
)]
mod old;
mod results;
mod typed;
