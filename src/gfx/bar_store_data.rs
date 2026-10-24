//! The progress bar's variables (position, width, glyphs, percent) first values: the bytes of RAL.EXE's 0x755224 .. 0x75523C that are not zero (generated from the image; `bar_store::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[];
