//! `race::blit_map_tiles` (0x41B931): the map's tiles into its picture: the texture eax
//! locked (`d3d::lock_texture`; carry: nothing), then 8 x 8 tiles of 16 x 16 pixels from
//! tile (ecx, edx) on: each the tile's pixels (`g_mapTiles`, 0x80 x 0x80 pointers, 16 x 16
//! pixels each) or, for a tile without them or outside the 0x80 x 0x80, the colour
//! `g_mapBlack`, as words on a surface of 2 bytes a pixel and as dwords on any other, at the
//! tile's place on the locked surface (`g_surface`); the gap between a tile's rows kept in
//! `dword_5BE9AD`; unlocked (`d3d::unlock_texture`, eax as the lock left it).
//!
//! In:  eax = the texture, ecx / edx = the first tile's column and row
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `race::map_tiles_stub`, so `race::map_tiles` (the map's scroll). It begins with
//! a call: a private copy (its two calls relocated) and a manual patch. The lock and the
//! unlock through `part` (0, 1; in the game through the game: the platform's, section 15).
//! Refuses (after the lock, unlocking first) a null surface, and a null tile table with a
//! tile inside the 0x80 x 0x80. Shadow mode runs the original alone (it tells the device).
//! Self-test: a copy whose two calls go to stand-ins (the lock failing or not), against the
//! Rust version, on a surface and tiles of the test's: 2 and 4 bytes a pixel, tiles inside,
//! across the edge and outside.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use rc2k_core::race::map::TILE;

const BYTES_RVA: usize = rva(0x4F49AC); // g_surface.bytes_per_pixel
const PITCH_RVA: usize = rva(0x4F49BC); // g_surface.pitch
const PIXELS_RVA: usize = rva(0x4F49C0); // g_surface.pixels
const TILES_RVA: usize = rva(0x5BE991); // g_mapTiles
const BLACK_RVA: usize = rva(0x5BE9B1); // g_mapBlack
const GAP_RVA: usize = rva(0x5BE9AD); // dword_5BE9AD
const LOCK_RVA: usize = rva(0x41AA4B); // d3d::lock_texture
const UNLOCK_RVA: usize = rva(0x41AB02); // d3d::unlock_texture

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn blit_map_tiles(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    crate::race::blit_map_tiles::game_part(0, &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    // Converted: `rc2k_core::race::map::{any_inside, check_tiles, row_gap, tile_offset,
    // tile_index, pixel_width}`.
    use rc2k_core::race::map::{
        BLITTED, any_inside, check_tiles, pixel_width, row_gap, tile_index, tile_offset,
    };
    let (bytes, pitch, pixels) = (g(BYTES_RVA), g(PITCH_RVA), g(PIXELS_RVA));
    let tiles = g(TILES_RVA);
    let (first_x, first_y) = (entry.ecx, entry.edx);
    if let Err(e) = check_tiles(pixels, tiles, any_inside(first_x, first_y)) {
        let mut u = r;
        crate::race::blit_map_tiles::game_part(1, &mut u)?;
        return Err(e.into());
    }
    let gap = row_gap(pitch, bytes);
    unsafe { game_write(GAP_RVA, gap) };
    let black = g(BLACK_RVA);
    // The registers the loops leave for the unlock: esi and edi where the last tile left
    // them, ebp the gap, ebx the rows done, ecx back at the first column, edx past the rows.
    let (mut last_from, mut last_to) = (0u32, 0u32);
    for row in 0..BLITTED {
        for col in 0..BLITTED {
            let (x, y) = (first_x.wrapping_add(col), first_y.wrapping_add(row));
            let mut to = pixels.wrapping_add(tile_offset(col, row, bytes, pitch));
            let tile = match tile_index(x, y) {
                Some(n) => unsafe { ((tiles + 4 * n) as *const u32).read_unaligned() },
                None => 0,
            };
            let wide = pixel_width(bytes);
            let mut from = tile;
            for _ in 0..TILE {
                for _ in 0..TILE {
                    unsafe {
                        if wide == 2 {
                            let v = if tile != 0 {
                                (from as *const u16).read_unaligned()
                            } else {
                                black as u16
                            };
                            (to as *mut u16).write_unaligned(v);
                        } else {
                            let v = if tile != 0 {
                                (from as *const u32).read_unaligned()
                            } else {
                                black
                            };
                            (to as *mut u32).write_unaligned(v);
                        }
                    }
                    to = to.wrapping_add(wide);
                    from = from.wrapping_add(wide);
                }
                to = to.wrapping_add(gap);
            }
            (last_from, last_to) = (if tile != 0 { from } else { 0 }, to);
        }
    }
    r.esi = last_from;
    r.edi = last_to;
    r.ebp = gap;
    r.ebx = BLITTED;
    r.ecx = first_x;
    r.edx = first_y.wrapping_add(BLITTED);
    crate::race::blit_map_tiles::game_part(1, &mut r)?;
    Ok(*entry)
}

/// In the game: the platform's lock and unlock through the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::calls::call(if place == 0 { LOCK_RVA } else { UNLOCK_RVA }, r);
    Ok(())
}

/// The Rust version with the game's calls: for Rust callers.
pub unsafe fn blit_map_tiles_in_game(r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { blit_map_tiles(r) }?;
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { blit_map_tiles(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
