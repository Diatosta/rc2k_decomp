//! The record seal's cipher work (`race::save_seal`, `records::seal_time`): the key, the block
//! being enciphered, its result and the sixteen subkeys. The cipher's tables are constants in
//! `race::seal_tables`.

use crate::session::{PlainBytes, PlainDword};

/// The seal cipher's subkeys, key, block and result.
#[derive(rc2k_derive::Cells)]
pub struct SealWork {
    /// Padding before the first field.
    pub _lead: PlainBytes<12>,
    /// Sixteen subkeys (two dwords each), made from the key by every block.
    pub subkeys: [PlainDword; 32],
    /// The key (two dwords; the `key` setting of `var\ral.cfg` sets it).
    pub key: [PlainDword; 2],
    /// The block to encipher (two dwords).
    pub block: [PlainDword; 2],
    /// The enciphered block (two dwords).
    pub sealed: [PlainDword; 2],
    /// Padding after the last field.
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
