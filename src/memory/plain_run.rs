//! Runs of the plain words of a record laid end to end, as the game's code overran one field into
//! the next: a word (`u32`, `i32`, `f32`), or an array of runs.

/// A run of 32-bit words in a record.
pub trait PlainRun {
    /// How many words the run holds.
    fn count(&self) -> usize;
    /// Word `i`, as bits.
    fn bits(&self, i: usize) -> u32;
    /// Word `i` put in, as bits.
    fn set_bits(&mut self, i: usize, v: u32);
}

impl PlainRun for u32 {
    fn count(&self) -> usize {
        1
    }
    fn bits(&self, _: usize) -> u32 {
        *self
    }
    fn set_bits(&mut self, _: usize, v: u32) {
        *self = v;
    }
}

impl PlainRun for i32 {
    fn count(&self) -> usize {
        1
    }
    fn bits(&self, _: usize) -> u32 {
        *self as u32
    }
    fn set_bits(&mut self, _: usize, v: u32) {
        *self = v as i32;
    }
}

impl PlainRun for f32 {
    fn count(&self) -> usize {
        1
    }
    fn bits(&self, _: usize) -> u32 {
        self.to_bits()
    }
    fn set_bits(&mut self, _: usize, v: u32) {
        *self = f32::from_bits(v);
    }
}

impl<R: PlainRun, const N: usize> PlainRun for [R; N] {
    fn count(&self) -> usize {
        self.first().map_or(0, |each| N * each.count())
    }
    fn bits(&self, i: usize) -> u32 {
        let each = self[0].count();
        self[i / each].bits(i % each)
    }
    fn set_bits(&mut self, i: usize, v: u32) {
        let each = self[0].count();
        self[i / each].set_bits(i % each, v);
    }
}
