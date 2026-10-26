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

use crate::framework::Regs;
use crate::install::installer::{Api, GET_DISK_FREE, SCRATCH, a, rd};

/// The registers and flags it leaves.
pub fn free_space_regs(entry: &Regs, api: Api) -> Regs {
    let mut r = *entry;
    let root = unsafe { crate::files::drive_type::drive_root(r.eax as u8) };
    let root_ptr = root.as_ref().map_or(0, |b| b.as_ptr() as u32);
    let s = a(SCRATCH);
    let answer = api(GET_DISK_FREE, &[root_ptr, s + 0xC, s + 8, s + 4, s]);
    if answer < 1 {
        r.eax = u32::MAX;
        r.set_carry(true);
        r.set_zero(false);
        return r;
    }
    r.ebx = rd(s + 8);
    // Converted: `rc2k_core::files::log::free_kb`.
    r.eax = rc2k_core::files::log::free_kb(rd(s + 0xC), rd(s + 8), rd(s + 4));
    r.set_carry(false);
    r.set_zero(r.eax == 0);
    r
}
