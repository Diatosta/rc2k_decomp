//! `drive::pack_inputs` (0x409328): four input words packed into one: eax | ((ecx << 9 |
//! edx) << 9), << 4 (as the original's `shl`s, high bits lost).
//!
//! In:  eax, ecx, edx the words (ebx not used)
//! Out: eax the packed word; the rest kept; no flags read
//!
//! Caller: `drive::record_inputs`.
//! Self-test: the original and the Rust version on words by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

/// The Rust version.
// Converted: `rc2k_core::drive::inputs::pack_inputs`.
pub use rc2k_core::drive::inputs::pack_inputs;
