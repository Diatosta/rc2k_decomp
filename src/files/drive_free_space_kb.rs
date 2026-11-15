//! A drive's free space in KB: the drive's letter in the low byte (0: the current drive), its
//! root `x:\` with the letter lowered; the KB are (sectors a cluster x bytes a sector, its low
//! 32 bits) x free clusters, shifted down 10 bits.

use crate::system::owner::System;

/// The free KB of drive `letter` (its low byte), or None when the query failed.
pub fn drive_free_space_kb(system: &System, letter: u32) -> Option<u32> {
    let root = crate::files::drive_type::drive_root(letter as u8);
    let text = root.as_ref().map(|r| {
        let n = r.iter().position(|&c| c == 0).unwrap_or(r.len());
        &r[..n]
    });
    // Sectors a cluster, bytes a sector, free clusters, total clusters.
    let [sectors, bytes, free, total] = crate::platform::os::disk_free(text)?;
    for (k, v) in [total, free, bytes, sectors].into_iter().enumerate() {
        system.work().files.buffer.set_u32(4 * k, v);
    }
    Some(rc2k_core::files::log::free_kb(sectors, bytes, free))
}
