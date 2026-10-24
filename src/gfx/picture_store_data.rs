//! The picture loader's variables (the name, flags, size, rows and buffer of the picture, the list, the grades) first values: the bytes of RAL.EXE's 0x4F49C8 .. 0x4F4B30 that are not zero (generated from the image; `picture_store::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[(
    0x5c,
    &[0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01],
)];
