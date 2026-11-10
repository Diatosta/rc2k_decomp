//! `drive::unpack_inputs` (0x409338): a packed input word (`drive::pack_inputs`) taken apart: eax
//! bits 4 .. 12, edx bits 13 .. 21, ecx the top ten (`sar` 22).
//!
//! Takes: the packed word
//! Returns: the words
//!
//! Callers: `drive::play_inputs`, `drive::record_inputs`.

/// [eax, ecx, edx].
// Converted: `rc2k_core::drive::inputs::unpack_inputs`.
pub use rc2k_core::drive::inputs::unpack_inputs;
