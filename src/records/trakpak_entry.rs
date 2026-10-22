//! `records::trakpak_entry` (0x473027): returns the entry of the loaded track pack that a
//! record refers to.
//!
//! In:  edi = record (dword +4 = entry index, dword +8 = track pack id)
//! Out: edi = esi = entry pointer, ebx = index * 0x40, ecx = the record's pack id,
//!      edx = the loaded pack's id; eax and ebp preserved
//! The entries are 0x40 bytes each and start 0x34 bytes into the pack header. If the record's
//! pack id is not the loaded pack's, the original reports "Invalid TrakPak Header" or, when the
//! header's first dword is non-zero, loops forever (it reloads the header address on every
//! pass instead of following the link): that case is left to the original.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::FatalError;
use crate::framework::{game_addr, game_read, rva};

// ---- game data ----

const HEADER_RVA: usize = rva(0x734BC6); // g_trakPakHeader: +0 link, +4 pack id, +0x34 entries

pub fn loaded_pack_id() -> u32 {
    unsafe { game_read(HEADER_RVA + 4) }
}

// ---- Rust version ----

/// What the original reports when a record belongs to another track pack.
pub const INVALID_HEADER: FatalError = FatalError {
    code: 0x8070_DA00,
    message: "Invalid TrakPak Header",
};

/// Address of entry `index` in the pack header at address `header`.
/// The original loops forever instead of reporting the error when the header's first dword
/// is non-zero; this version reports it in both cases.
pub fn trakpak_entry(
    header: u32,
    index: u32,
    pack_id: u32,
    loaded_pack_id: u32,
) -> Result<u32, FatalError> {
    // Converted: `rc2k_core::files::records::pack_entry`.
    rc2k_core::files::records::pack_entry(index, pack_id, loaded_pack_id)
        .map(|place| header.wrapping_add(place as u32))
        .ok_or(INVALID_HEADER)
}

/// The entry for the record at `record`, in the pack the game has loaded.
pub unsafe fn record_entry(record: *const u8) -> Result<u32, FatalError> {
    let (index, pack_id) = unsafe {
        (
            record.add(4).cast::<u32>().read_unaligned(),
            record.add(8).cast::<u32>().read_unaligned(),
        )
    };
    trakpak_entry(
        game_addr(HEADER_RVA) as u32,
        index,
        pack_id,
        loaded_pack_id(),
    )
}
