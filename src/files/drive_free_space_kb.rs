//! `files::drive_free_space_kb` (0x40317D): a drive's free space in KB.
//!
//! In:  eax = the drive's letter in its low byte (0: the current drive)
//! Out: eax = the free KB, or -1 when `GetDiskFreeSpaceA` failed (carry set) or the KB do
//!      not fit 32 bits (carry clear, as for an answer); else carry clear and zero by eax.
//!      After an answer ebx = the bytes a sector; ecx, edx, esi kept
//! The root asked about is `x:\` with the letter lowered (`files::drive_root`), and the
//! four figures go to the first 16 bytes of `g_pngInfo` (total clusters, free clusters,
//! bytes a sector, sectors a cluster). The KB are (sectors a cluster x bytes a sector, its
//! low 32 bits) x free clusters, shifted down 10 bits; past 2^42 bytes it gives -1.
//! Callers: the installer (`install::install_files`, `install::list_drives`) and
//! `files::free_space_kb`. The Win32 call through `api` (`installer::game_api` in the
//! game).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::files::scratch::SCRATCH;
use crate::install::installer::{Api, GET_DISK_FREE, rd};

/// The free KB of drive `letter` (its low byte), or None when `GetDiskFreeSpaceA` failed
/// (the original's carry; eax -1 then). The bytes a sector the original leaves in ebx are
/// read by no caller.
pub fn drive_free_space_kb(letter: u32, api: Api) -> Option<u32> {
    let root = crate::files::drive_type::drive_root(letter as u8);
    let root_ptr = root
        .as_ref()
        .map_or(0, |b| crate::pointers::game_address(b.as_ptr()));
    let s = SCRATCH.buffer.addr();
    let answer = api(GET_DISK_FREE, &[root_ptr, s + 0xC, s + 8, s + 4, s]);
    if answer < 1 {
        return None;
    }
    // Converted: `rc2k_core::files::log::free_kb`.
    Some(rc2k_core::files::log::free_kb(
        rd(s + 0xC),
        rd(s + 8),
        rd(s + 4),
    ))
}
