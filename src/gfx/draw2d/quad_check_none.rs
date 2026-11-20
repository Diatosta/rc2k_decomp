//! The handheld's side of the quad check (`quad_check.rs`): it has no such check, and nothing is
//! kept.

/// Nothing is kept.
pub struct Interval;

impl Interval {
    pub const fn new() -> Interval {
        Interval
    }

    /// Nothing to note.
    pub fn note(&mut self, _screen_len: usize, _at: usize, _bytes: &[u8]) {}
}
