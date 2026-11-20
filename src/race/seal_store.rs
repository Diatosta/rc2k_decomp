//! The record seal's cipher work (`race::save_seal`, `records::seal_time`): the key, the block
//! being enciphered, its result and the sixteen subkeys. The cipher's tables are constants in
//! `race::seal_tables`.

/// The seal cipher's subkeys, key, block and result.
pub struct SealWork {
    /// Sixteen subkeys (two dwords each), made from the key by every block.
    pub subkeys: [u32; 32],
    /// The key (two dwords; the `key` setting of `var\ral.cfg` sets the first).
    pub key: [u32; 2],
    /// The block to encipher (two dwords).
    pub block: [u32; 2],
    /// The enciphered block (two dwords).
    pub sealed: [u32; 2],
}

impl SealWork {
    pub const fn new() -> Self {
        SealWork {
            subkeys: [0; 32],
            key: [0x6b726164, 0x72617473],
            block: [0; 2],
            sealed: [0; 2],
        }
    }
}
