//! `system::next_entry` (0x4017E5): one step on in a table of 8-byte entries.
//!
//! In:  ebx the entry, ecx the entries left with it
//! Out: ecx 0: carry set, nothing changed. Else ebx + 8 and ecx - 1; carry set if that
//!      leaves 0, else the carry of the `add ebx, 8` (`dec` keeps it: clear unless ebx
//!      wrapped). Everything else preserved.
//! Callers: `gfx::mark_3d_modes` (which does not read the carry) and 0x404718. Manual patch
//! with a private copy (0xD bytes): the `jb` at +3 is in the first bytes. Shadow mode:
//! both, compared. Self-test: the original against the Rust version on counts and places
//! (one that wraps).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

// ---- Rust version ----

/// (carry, ebx, ecx).
pub fn next_entry(entry: u32, left: u32) -> (bool, u32, u32) {
    // Converted: `rc2k_core::system::tables::next`.
    rc2k_core::system::tables::next(entry, left)
}
