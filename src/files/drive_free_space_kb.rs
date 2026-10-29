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
//! `files::free_space_kb`. The Win32 call is the system's (`os::disk_free`).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::files::scratch::SCRATCH;

/// The free KB of drive `letter` (its low byte), or None when `GetDiskFreeSpaceA` failed
/// (the original's carry; eax -1 then). The bytes a sector the original leaves in ebx are
/// read by no caller.
pub fn drive_free_space_kb(letter: u32) -> Option<u32> {
    let root = crate::files::drive_type::drive_root(letter as u8);
    let text = root.as_ref().map(|r| {
        let n = r.iter().position(|&c| c == 0).unwrap_or(r.len());
        &r[..n]
    });
    // Sectors a cluster, bytes a sector, free clusters, total clusters.
    let [sectors, bytes, free, total] = crate::os::disk_free(text)?;
    for (k, v) in [total, free, bytes, sectors].into_iter().enumerate() {
        SCRATCH.buffer.set_u32(4 * k, v);
    }
    // Converted: `rc2k_core::files::log::free_kb`.
    Some(rc2k_core::files::log::free_kb(sectors, bytes, free))
}
