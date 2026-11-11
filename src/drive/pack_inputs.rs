//! `drive::pack_inputs` (0x409328): four input words packed into one: the first |, << 4 (as the
//! original's `shl`s, high bits lost).
//!
//! Takes: the words (not used)
//! Returns: the packed word
//!
//! Caller: `drive::record_inputs`.

// Converted: `rc2k_core::drive::inputs::pack_inputs`.
pub use rc2k_core::drive::inputs::pack_inputs;
