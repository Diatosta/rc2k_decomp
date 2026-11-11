//! `drive::unpack_inputs` (0x409338): a packed input word (`drive::pack_inputs`) taken apart: the
//! first word is bits 4 .. 12, the second bits 13 .. 21, the third the top ten (arithmetic shift 22).
//!
//! Takes: the packed word
//! Returns: the words
//!
//! Callers: `drive::play_inputs`, `drive::record_inputs`.

/// The three words.
// Converted: `rc2k_core::drive::inputs::unpack_inputs`.
pub use rc2k_core::drive::inputs::unpack_inputs;
