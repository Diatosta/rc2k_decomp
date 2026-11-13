//! The record seal's cipher work (`race::save_seal`, `race::replay`, `records::seal_time`),
//! out of RAL.EXE's image (phase 5), typed (phase 7): the key, the block being enciphered, its
//! result and the sixteen subkeys. The cipher's tables (the image's read-only bytes) are
//! Rust constants (`race::seal_tables`).
//!
//! A struct of cells with the image's first values
//! (`tests`): the settings
//! table names the key (`key` in `var\ral.cfg`) by its IDA address.

use crate::session::{PlainBytes, PlainDword};

/// The seal cipher's subkeys, key, block and result (IDA 0x4E04AC .. 0x4E0544, the image's layout).
#[derive(rc2k_derive::Cells)]
pub struct SealWork {
    /// Bytes before the first field, not in the image.
    pub _lead: PlainBytes<12>,
    /// 0x4E04AC: `dword_4E04AC`: sixteen subkeys (two dwords each), made from the key by every block.
    pub subkeys: [PlainDword; 32],
    /// 0x4E052C: `dword_4E052C`: the key (two dwords; the `key` setting of `var\ral.cfg` sets it).
    pub key: [PlainDword; 2],
    /// 0x4E0534: `dword_4E0534`: the block to encipher (two dwords).
    pub block: [PlainDword; 2],
    /// 0x4E053C: `value`, `dword_4E0540`: the enciphered block (two dwords).
    pub sealed: [PlainDword; 2],
    /// Bytes after the last field, not in the image.
    pub _tail: PlainBytes<12>,
}

impl SealWork {
    pub const fn new() -> Self {
        SealWork {
            _lead: PlainBytes::new([0; 12]),
            subkeys: [const { PlainDword::new(0x0) }; 32],
            key: [PlainDword::new(0x6b726164), PlainDword::new(0x72617473)],
            block: [const { PlainDword::new(0x0) }; 2],
            sealed: [const { PlainDword::new(0x0) }; 2],
            _tail: PlainBytes::new([0; 12]),
        }
    }
}
