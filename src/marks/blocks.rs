//! More of `marks::init_marks`' steps (IDA, 2026-10-16): the three blocks it takes
//! (`memory::allocate`), the corner table
//! scaled, and the eight settings blocks reset.

use crate::errors::Refused;
use crate::framework::rva;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;

pub const PLACES_LIST_RVA: Slot = Slot::at(0x8DB570); // stru_8DB570
pub const PLACES_RVA: Slot = Slot::at(0x8DB5B0); // g_marksPlaces
pub const PLACES_LEN: u32 = 0x7024;
const PLACE_START_RVA: Slot = Slot::at(0x8DB5B4); // flt_8DB5B4 .. flt_8DB5BC
pub const RING_LIST_RVA: Slot = Slot::at(0x93A580); // stru_93A580
pub const RING_RVA: Slot = Slot::at(0x93A5C8); // g_marksRing
pub const RING_FREE_RVA: Slot = Slot::at(0x93A5C0); // g_marksRingFree
pub const RING_USED_RVA: Slot = Slot::at(0x93A5EC); // g_marksRingUsed
pub const RING_LEN: u32 = 0x4C000;
pub const RING_ENTRY: u32 = rc2k_core::marks::rings::RING_ENTRY;
pub const LINKS_LIST_RVA: Slot = Slot::at(0x9206F0); // stru_9206F0
pub const LINKS_RVA: Slot = Slot::at(0x920730); // g_marksLinks
pub const LINKS_LEN: u32 = 0x71020;
pub const CORNERS_RVA: Slot = Slot::at(0x8EB971); // g_marksCorners: 0x100 of 0x80, words
pub const CORNERS: u32 = 0x100;
const CORNER_SCALE_RVA: Slot = Slot::at(0x8EB969); // flt_8EB969, 1 / 65536
/// Where the corners' eight values are made (two dwords each at +0, +0x20, +0x40, +0x60).
pub const CORNER_WORK_RVA: usize = rva(0x9605B8);
pub const CORNER_SIDES_RVA: Slot = Slot::at(0x8EB8B5); // dword_8EB8B5, dword_8EB8B9
pub const SLOPES_RVA: Slot = Slot::at(0x920734); // g_marksCornerSlopes: 0x100 of 0x24
pub const SETTINGS_BLOCKS: u32 = 8;
pub const SETTINGS_BLOCK: u32 = 0x4C8;
const SETTINGS_START_RVA: Slot = Slot::at(0x9354C6); // flt_9354C6 .. flt_9354DA

/// `bytes` bytes on the list at `list` (`memory::allocate`).
pub unsafe fn take(list: Slot, bytes: u32) -> Result<u32, Refused> {
    let list = game_addr(list) as *mut crate::memory::MemoryList;
    Ok(unsafe { crate::memory::allocate::allocate(list, bytes) }? as u32)
}

/// `marks::allocate_places` (0x4C38B1), step 6: 0x7024 bytes on `stru_8DB570` into
/// `g_marksPlaces`, its first three floats `flt_8DB5B4` ...
pub unsafe fn allocate_places() -> Result<(), Refused> {
    unsafe {
        let block = take(PLACES_LIST_RVA, PLACES_LEN)?;
        game_write(PLACES_RVA, block);
        for n in 0..3 {
            let v = x87::loaded(game_read(PLACE_START_RVA + 4 * n));
            ((block + 4 * n as u32) as *mut u32).write_unaligned(v);
        }
    }
    Ok(())
}

/// `marks::allocate_ring` (0x4CFAC5), step 13: 0x4C000 bytes on `stru_93A580` into
/// `g_marksRing` and `g_marksRingFree`, `g_marksRingUsed` 0, and the 0x1000 entries of
/// 0x4C made a ring (+0 the one before, +4 the one after).
pub unsafe fn allocate_ring() -> Result<(), Refused> {
    unsafe {
        let block = take(RING_LIST_RVA, RING_LEN)?;
        game_write(RING_RVA, block);
        game_write(RING_FREE_RVA, block);
        game_write(RING_USED_RVA, 0u32);
        // Converted: `rc2k_core::marks::rings::ring_links`.
        for (n, links) in rc2k_core::marks::rings::ring_links(block).enumerate() {
            let e = block.wrapping_add(n as u32 * RING_ENTRY) as *mut [u32; 2];
            e.write_unaligned(links);
        }
    }
    Ok(())
}

/// `marks::allocate_links` (0x4C5398), in step 3: 0x71020 bytes on `stru_9206F0` into
/// `g_marksLinks`, its +0x7100C 0, at +8 and +0x4008 the 0x1000 entries' before and after
/// (round), then `marks::scale_corners`.
pub unsafe fn allocate_links() -> Result<(), Refused> {
    unsafe {
        let block = take(LINKS_LIST_RVA, LINKS_LEN)?;
        game_write(LINKS_RVA, block);
        ((block + 0x7100C) as *mut u32).write_unaligned(0);
        // Converted: `rc2k_core::marks::rings::link_tables`.
        let (before, after) = rc2k_core::marks::rings::link_tables();
        ((block + 8) as *mut [u32; 0x1000]).write_unaligned(before);
        ((block + 0x4008) as *mut [u32; 0x1000]).write_unaligned(after);
        scale_corners();
    }
    Ok(())
}

/// The word pairs of a corner record, in the order they are made: +0, +4, +0xC, +8.
const CORNER_PAIRS: [u32; 4] = [0, 4, 0xC, 8];

/// `marks::scale_corners` (0x4C6420): for the 0x100 corner records (0xFF first), the four
/// word pairs (+0, +4, +0xC, +8) each times `flt_8EB969` into the work and into +4 .. +0x20
/// of the record's 0x24 at `g_marksCornerSlopes`; +0 there the slope: |+0xA - +2| over
/// |+4 - +0| (sixteen-bit differences; `dword_8EB8B5`, `dword_8EB8B9`).
pub unsafe fn scale_corners() {
    unsafe {
        let scale: u32 = game_read(CORNER_SCALE_RVA);
        for n in (0..CORNERS).rev() {
            let record = game_addr(CORNERS_RVA) as u32 + 0x80 * n;
            let word = |o: u32| ((record + o) as *const u16).read_unaligned();
            let mut values = [0u32; 8];
            for (k, &o) in CORNER_PAIRS.iter().enumerate() {
                for h in 0..2 {
                    let v = x87::int_times(word(o + 2 * h as u32) as i32, scale);
                    values[2 * k + h] = v;
                    game_write(CORNER_WORK_RVA + 0x20 * k + 4 * h, v);
                }
            }
            let side = |a: u32, b: u32| {
                let d = word(a).wrapping_sub(word(b));
                (if (d as i16) < 0 { d.wrapping_neg() } else { d }) as u32
            };
            let (across, along) = (side(4, 0), side(0xA, 2));
            game_write(CORNER_SIDES_RVA, [across, along]);
            let slope = SLOPES_RVA + (n * 0x24) as usize;
            game_write(slope, x87::int_ratio(along as i32, across as i32));
            game_write(slope + 4, values);
        }
    }
}

/// `marks::reset_settings_blocks` (0x4C9A6D), in step 3: the eight blocks of 0x4C8 at
/// `g_marksSettings` + 0x60 (the last first): +0x4C .. +0x60 the six floats from
/// `flt_9354C6`, the last three again at +0x64, +0x70, +0x7C, +0x30 8.
pub unsafe fn reset_settings_blocks() {
    unsafe {
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        for n in (0..SETTINGS_BLOCKS).rev() {
            let block = settings.wrapping_add(0x60 + n * SETTINGS_BLOCK);
            let at = |o: u32| (block + o) as *mut u32;
            for k in 0..6u32 {
                let v = x87::loaded(game_read(SETTINGS_START_RVA + 4 * k as usize));
                at(0x4C + 4 * k).write_unaligned(v);
            }
            let last: [u32; 3] = [0x58, 0x5C, 0x60].map(|o| at(o).read_unaligned());
            for o in [0x64, 0x70, 0x7C] {
                for (k, v) in last.into_iter().enumerate() {
                    at(o + 4 * k as u32).write_unaligned(v);
                }
            }
            at(0x30).write_unaligned(8);
        }
    }
}
