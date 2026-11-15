//! Builders for the first values of the race block's stores: arrays of cells from plain
//! numbers or a text, as constants.

use crate::memory::session::{Dword, PlainDword};

macro_rules! cells_of {
    ($name:ident, $cell:ident, $item:ty) => {
        /// The cells holding `values`.
        pub const fn $name<const N: usize>(values: [$item; N]) -> [$cell; N] {
            let mut out: [$cell; N] = [const { $cell::new(0) }; N];
            let mut k = 0;
            while k < N {
                out[k] = $cell::new(values[k]);
                k += 1;
            }
            out
        }
    };
}

macro_rules! cells_of_text {
    ($name:ident, $cell:ident) => {
        /// The dwords of a text, zero-padded to `N` dwords: the bytes in the game's order.
        pub const fn $name<const N: usize>(text: &[u8]) -> [$cell; N] {
            assert!(text.len() <= 4 * N);
            let mut out: [$cell; N] = [const { $cell::new(0) }; N];
            let mut k = 0;
            while k < N {
                let mut word = [0u8; 4];
                let mut b = 0;
                while b < 4 && 4 * k + b < text.len() {
                    word[b] = text[4 * k + b];
                    b += 1;
                }
                out[k] = $cell::new(u32::from_le_bytes(word));
                k += 1;
            }
            out
        }
    };
}

cells_of!(dwords, Dword, u32);
cells_of!(plain_dwords, PlainDword, u32);
cells_of_text!(plain_dwords_of_text, PlainDword);
