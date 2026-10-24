//! The PNG header's fields (width, height, compression, colour type) first values: the bytes of RAL.EXE's 0xA8C190 .. 0xA8C1A4 that are not zero (generated from the image; `png_header::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[];
