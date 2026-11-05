//! The record seal's cipher work (`race::save_seal`, `race::replay`, `records::seal_time`),
//! out of RAL.EXE's image (phase 5), typed (phase 7): the key, the block being enciphered, its
//! result and the sixteen subkeys. The cipher's tables (the image's read-only bytes) are
//! Rust constants (`race::seal_tables`).
//!
//! A `#[repr(C)]` struct of atomic cells with the image's layout, first values the image's
//! (`tests`); `moved::MOVED` points the image's addresses at it (`home`): the settings
//! table names the key (`key` in `var\ral.cfg`) by its IDA address.

use crate::session::{Bytes, Dword};

/// The seal cipher's subkeys, key, block and result (IDA 0x4E04AC .. 0x4E0544, the image's layout).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct SealWork {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: Bytes<12>,
    /// 0x4E04AC: `dword_4E04AC`: sixteen subkeys (two dwords each), made from the key by every block.
    pub subkeys: [Dword; 32],
    /// 0x4E052C: `dword_4E052C`: the key (two dwords; the `key` setting of `var\ral.cfg` sets it).
    pub key: [Dword; 2],
    /// 0x4E0534: `dword_4E0534`: the block to encipher (two dwords).
    pub block: [Dword; 2],
    /// 0x4E053C: `value`, `dword_4E0540`: the enciphered block (two dwords).
    pub sealed: [Dword; 2],
    /// Not in the image: the struct's tail up to its 16-byte alignment.
    pub _tail: Bytes<12>,
}

pub static SEAL_WORK: SealWork = SealWork {
    _lead: Bytes::new([0; 12]),
    subkeys: [const { Dword::new(0x0) }; 32],
    key: [Dword::new(0x6b726164), Dword::new(0x72617473)],
    block: [const { Dword::new(0x0) }; 2],
    sealed: [const { Dword::new(0x0) }; 2],
    _tail: Bytes::new([0; 12]),
};

/// The part from IDA's `START` on (past the alignment lead): `moved::MOVED`'s home.
pub static SEAL_WINDOW: crate::race::store::Window<SealWork> =
    crate::race::store::Window::new(&SEAL_WORK, 12);

impl SealWork {
    pub const START: usize = 0x4E04AC;
    pub const LEN: usize = 0x98;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sealwork_starts_as_the_games() {
        let size = size_of::<SealWork>();
        assert!(12 + SealWork::LEN <= size && size <= (12 + SealWork::LEN).next_multiple_of(16));
        let image = crate::moved::image_bytes(SealWork::START, SealWork::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(crate::moved::place(SealWork::START)).cast_const(),
                SealWork::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
