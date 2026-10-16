//! More of `marks::init_marks`' steps (IDA, 2026-10-16): the three blocks it takes
//! (`memory::allocate`, as place 0 of a `race::level_init::Part`), the corner table
//! scaled, and the eight settings blocks reset.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;

pub const PLACES_LIST_RVA: usize = rva(0x8DB570); // stru_8DB570
pub const PLACES_RVA: usize = rva(0x8DB5B0); // g_marksPlaces
pub const PLACES_LEN: u32 = 0x7024;
const PLACE_START_RVA: usize = rva(0x8DB5B4); // flt_8DB5B4 .. flt_8DB5BC
pub const RING_LIST_RVA: usize = rva(0x93A580); // stru_93A580
pub const RING_RVA: usize = rva(0x93A5C8); // g_marksRing
pub const RING_FREE_RVA: usize = rva(0x93A5C0); // g_marksRingFree
pub const RING_USED_RVA: usize = rva(0x93A5EC); // g_marksRingUsed
pub const RING_LEN: u32 = 0x4C000;
pub const RING_ENTRIES: u32 = 0x1000;
pub const RING_ENTRY: u32 = 0x4C;
pub const LINKS_LIST_RVA: usize = rva(0x9206F0); // stru_9206F0
pub const LINKS_RVA: usize = rva(0x920730); // g_marksLinks
pub const LINKS_LEN: u32 = 0x71020;
pub const CORNERS_RVA: usize = rva(0x8EB971); // g_marksCorners: 0x100 of 0x80, words
pub const CORNERS: u32 = 0x100;
const CORNER_SCALE_RVA: usize = rva(0x8EB969); // flt_8EB969, 1 / 65536
/// Where the corners' eight values are made (two dwords each at +0, +0x20, +0x40, +0x60).
pub const CORNER_WORK_RVA: usize = rva(0x9605B8);
pub const CORNER_SIDES_RVA: usize = rva(0x8EB8B5); // dword_8EB8B5, dword_8EB8B9
pub const SLOPES_RVA: usize = rva(0x920734); // g_marksCornerSlopes: 0x100 of 0x24
pub const SETTINGS_RVA: usize = rva(0x767A28); // g_marksSettings
pub const SETTINGS_BLOCKS: u32 = 8;
pub const SETTINGS_BLOCK: u32 = 0x4C8;
const SETTINGS_START_RVA: usize = rva(0x9354C6); // flt_9354C6 .. flt_9354DA

unsafe fn take(
    g: &mut Regs,
    list: usize,
    bytes: u32,
    part: crate::race::level_init::Part,
) -> Result<u32, Refused> {
    g.eax = bytes;
    g.ebx = game_addr(list) as u32;
    part(0, g)?;
    Ok(g.eax)
}

/// `marks::allocate_places` (0x4C38B1), step 6: 0x7024 bytes on `stru_8DB570` into
/// `g_marksPlaces`, its first three floats `flt_8DB5B4` ... Leaves eax and esi the block,
/// ebx the list.
pub unsafe fn allocate_places(
    entry: &Regs,
    part: crate::race::level_init::Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let block = take(&mut g, PLACES_LIST_RVA, PLACES_LEN, part)?;
        game_write(PLACES_RVA, block);
        g.esi = block;
        for n in 0..3 {
            let v = x87::loaded(game_read(PLACE_START_RVA + 4 * n));
            ((block + 4 * n as u32) as *mut u32).write_unaligned(v);
        }
    }
    Ok(g)
}

/// `marks::allocate_ring` (0x4CFAC5), step 13: 0x4C000 bytes on `stru_93A580` into
/// `g_marksRing` and `g_marksRingFree`, `g_marksRingUsed` 0, and the 0x1000 entries of
/// 0x4C made a ring (+0 the one before, +4 the one after). Leaves eax 0x1000, ebx
/// the entry 0xFFE, ecx and esi the block.
pub unsafe fn allocate_ring(
    entry: &Regs,
    part: crate::race::level_init::Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let block = take(&mut g, RING_LIST_RVA, RING_LEN, part)?;
        game_write(RING_RVA, block);
        game_write(RING_FREE_RVA, block);
        game_write(RING_USED_RVA, 0u32);
        g.esi = block;
        let at = |n: u32| block.wrapping_add((n & (RING_ENTRIES - 1)) * RING_ENTRY);
        for n in 0..RING_ENTRIES {
            g.eax = n * RING_ENTRY;
            g.ebx = at(n.wrapping_sub(1));
            g.ecx = at(n + 1);
            let e = block.wrapping_add(g.eax) as *mut u32;
            e.write_unaligned(g.ebx);
            e.add(1).write_unaligned(g.ecx);
        }
        // The loop's counter, read back for its last test.
        g.eax = RING_ENTRIES;
    }
    Ok(g)
}

/// `marks::allocate_links` (0x4C5398), in step 3: 0x71020 bytes on `stru_9206F0` into
/// `g_marksLinks`, its +0x7100C 0, at +8 and +0x4008 the 0x1000 entries' before and after
/// (round), then `marks::scale_corners` (place 1 of the part). Leaves the registers as
/// that does.
pub unsafe fn allocate_links(
    entry: &Regs,
    part: crate::race::level_init::Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let block = take(&mut g, LINKS_LIST_RVA, LINKS_LEN, part)?;
        game_write(LINKS_RVA, block);
        g.esi = block;
        ((block + 0x7100C) as *mut u32).write_unaligned(0);
        for n in 0..0x1000u32 {
            g.eax = n;
            g.ebx = n.wrapping_sub(1) & 0xFFF;
            g.ecx = (n + 1) & 0xFFF;
            ((block + 8 + 4 * n) as *mut u32).write_unaligned(g.ebx);
            ((block + 0x4008 + 4 * n) as *mut u32).write_unaligned(g.ecx);
        }
        g.eax = 0x1000;
        part(1, &mut g)?;
    }
    Ok(g)
}

/// The word pairs of a corner record, in the order they are made: +0, +4, +0xC, +8.
const CORNER_PAIRS: [u32; 4] = [0, 4, 0xC, 8];

/// `marks::scale_corners` (0x4C6420): for the 0x100 corner records (0xFF first), the four
/// word pairs (+0, +4, +0xC, +8) each times `flt_8EB969` into the work and into +4 .. +0x20
/// of the record's 0x24 at `g_marksCornerSlopes`; +0 there the slope: |+0xA - +2| over
/// |+4 - +0| (sixteen-bit differences; `dword_8EB8B5`, `dword_8EB8B9`). Leaves eax and ebx
/// the last pair, esi 0, ebp and edi the two sides.
pub unsafe fn scale_corners(entry: &Regs) -> Regs {
    let mut g = *entry;
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
            g.ebp = side(4, 0);
            g.edi = side(0xA, 2);
            game_write(CORNER_SIDES_RVA, [g.ebp, g.edi]);
            g.esi = n * 0x24;
            game_write(
                SLOPES_RVA + g.esi as usize,
                x87::int_ratio(g.edi as i32, g.ebp as i32),
            );
            game_write(SLOPES_RVA + g.esi as usize + 4, values);
            g.eax = values[6];
            g.ebx = values[7];
        }
    }
    g
}

/// `marks::reset_settings_blocks` (0x4C9A6D), in step 3: the eight blocks of 0x4C8 at
/// `g_marksSettings` + 0x60 (the last first): +0x4C .. +0x60 the six floats from
/// `flt_9354C6`, the last three again at +0x64, +0x70, +0x7C, +0x30 8. Leaves ebp the
/// settings, esi the first block, eax, ebx, ecx the three.
pub unsafe fn reset_settings_blocks(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        g.ebp = game_read(SETTINGS_RVA);
        for n in (0..SETTINGS_BLOCKS).rev() {
            g.esi = g.ebp.wrapping_add(0x60 + n * SETTINGS_BLOCK);
            let at = |o: u32| (g.esi + o) as *mut u32;
            for k in 0..6u32 {
                let v = x87::loaded(game_read(SETTINGS_START_RVA + 4 * k as usize));
                at(0x4C + 4 * k).write_unaligned(v);
            }
            g.eax = at(0x58).read_unaligned();
            g.ebx = at(0x5C).read_unaligned();
            g.ecx = at(0x60).read_unaligned();
            for o in [0x64, 0x70, 0x7C] {
                at(o).write_unaligned(g.eax);
                at(o + 4).write_unaligned(g.ebx);
                at(o + 8).write_unaligned(g.ecx);
            }
            at(0x30).write_unaligned(8);
        }
    }
    g
}

/// What `scale_corners` writes.
pub fn corner_regions() -> Vec<crate::service::harness::Region> {
    let a = |r: usize| game_addr(r) as u32;
    let mut r: Vec<_> = (0..4).map(|k| (a(CORNER_WORK_RVA + 0x20 * k), 8)).collect();
    r.push((a(CORNER_SIDES_RVA), 8));
    r.push((a(SLOPES_RVA), 0x24 * CORNERS as usize));
    r
}

/// In the game: place 0 `memory::allocate`, 1 `scale_corners`, 2 `allocate_links`, 3
/// `reset_settings_blocks`.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => return crate::race::level_init::game_part(0, r),
            1 => *r = scale_corners(r),
            2 => *r = allocate_links(r, &mut game_part)?,
            _ => *r = reset_settings_blocks(r),
        }
    }
    Ok(())
}

/// A stand-in handler for the allocations' tests: noted; place 0 gives `GIVES` with the
/// carry clear, place 1 runs `scale_corners`; the others are only noted.
pub fn give(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    match place {
        0 => r.set_carry(false),
        1 => *r = unsafe { scale_corners(r) },
        _ => {}
    }
}

/// The same as a part, for the Rust side.
pub fn test_give(place: u32, r: &mut Regs) -> Result<(), Refused> {
    give(place, r);
    Ok(())
}
