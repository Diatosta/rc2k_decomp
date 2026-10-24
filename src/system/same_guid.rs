//! `system::same_guid` (0x4016F9): whether two 16-byte GUIDs are the same.
//!
//! In:  eax, ebx = the two GUIDs
//! Out: zero set if they are the same (the flags of the last `cmp`: the first differing
//!      dword's, else the fourth's); every register preserved (ecx pushed and popped)
//! Compared as four dwords, the first difference ending it. Callers: the network's
//! (`net::find_saved_connection`, `net::choose_session`, `net::connection_found`,
//! `net::build_address`) and two of the input's (0x4083F4, 0x40953B). Shadow mode: both
//! run, the zero flag compared. Self-test: against the original on GUIDs that differ in
//! each dword and none.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

/// The Rust version: whether the GUIDs at the two addresses are the same.
pub fn same_guid(a: u32, b: u32) -> bool {
    // Converted: `rc2k_core::system::machine::same_guid`.
    let at = |base: u32| move |k: u32| unsafe { ((base + 4 * k) as *const u32).read_unaligned() };
    rc2k_core::system::machine::same_guid(at(a), at(b))
}
