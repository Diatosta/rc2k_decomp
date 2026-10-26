//! `drive::unpack_inputs` (0x409338): a packed input word (`drive::pack_inputs`) taken
//! apart: eax bits 4 .. 12, edx bits 13 .. 21, ecx the top ten (`sar` 22).
//!
//! In:  eax the packed word
//! Out: eax, ecx, edx the words; the rest kept; no flags read
//!
//! Callers: `drive::play_inputs`, `drive::record_inputs`.

/// The Rust version: [eax, ecx, edx].
// Converted: `rc2k_core::drive::inputs::unpack_inputs`.
pub use rc2k_core::drive::inputs::unpack_inputs;
