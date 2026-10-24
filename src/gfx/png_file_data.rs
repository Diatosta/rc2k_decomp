//! The PNG decoder's file (pointer, length, memory list) first values: the bytes of RAL.EXE's 0x5BEC30 .. 0x5BEC48 that are not zero (generated from the image; `png_file::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[];
