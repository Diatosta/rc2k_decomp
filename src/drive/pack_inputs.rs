//! `drive::pack_inputs` (0x409328): four input words packed into one: eax | ((ecx << 9 |
//! edx) << 9), << 4 (as the original's `shl`s, high bits lost).
//!
//! In:  eax, ecx, edx the words (ebx not used)
//! Out: eax the packed word; the rest kept; no flags read
//!
//! Caller: `drive::record_inputs`.

/// The Rust version.
// Converted: `rc2k_core::drive::inputs::pack_inputs`.
pub use rc2k_core::drive::inputs::pack_inputs;
