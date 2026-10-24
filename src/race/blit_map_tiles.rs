//! `race::blit_map_tiles` (0x41B931): the map's tiles into its picture: the texture eax
//! locked (`d3d::lock_texture`; carry: nothing), then 8 x 8 tiles of 16 x 16 pixels from
//! tile (ecx, edx) on: each the tile's pixels (`g_mapTiles`, 0x80 x 0x80 pointers, 16 x 16
//! pixels each) or, for a tile without them or outside the 0x80 x 0x80, the colour
//! `g_mapBlack`, as words on a surface of 2 bytes a pixel and as dwords on any other, at the
//! tile's place on the locked surface (`g_surface`); the gap between a tile's rows kept in
//! `dword_5BE9AD`; unlocked (`d3d::unlock_texture`, eax as the lock left it).
//!
//! In:  `texture` (eax), `first_x` / `first_y` (ecx / edx) the first tile's column and row
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `race::map_tiles_stub`, so `race::map_tiles` (the map's scroll). Refuses (after
//! the lock, unlocking first) a null surface, and a null tile table with a tile inside the
//! 0x80 x 0x80. Hook removed 2026-10-23 (conversion phase 4).

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use rc2k_core::race::map::TILE;

const BYTES_RVA: usize = rva(0x4F49AC); // g_surface.bytes_per_pixel
const PITCH_RVA: usize = rva(0x4F49BC); // g_surface.pitch
const PIXELS_RVA: usize = rva(0x4F49C0); // g_surface.pixels
const TILES_RVA: usize = rva(0x5BE991); // g_mapTiles
const BLACK_RVA: usize = rva(0x5BE9B1); // g_mapBlack
const GAP_RVA: usize = rva(0x5BE9AD); // dword_5BE9AD

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

/// The Rust version (see the header).
pub unsafe fn blit_map_tiles(texture: u32, first_x: u32, first_y: u32) -> Result<(), Refused> {
    use crate::d3d::{lock_texture::lock_texture, unlock_texture::unlock_texture};
    // Converted: `rc2k_core::race::map::{any_inside, check_tiles, row_gap, tile_offset,
    // tile_index, pixel_width}`.
    use rc2k_core::race::map::{
        BLITTED, any_inside, check_tiles, pixel_width, row_gap, tile_index, tile_offset,
    };
    if unsafe { lock_texture(texture) }?.carry {
        return Ok(());
    }
    let (bytes, pitch, pixels) = (g(BYTES_RVA), g(PITCH_RVA), g(PIXELS_RVA));
    let tiles = g(TILES_RVA);
    if let Err(e) = check_tiles(pixels, tiles, any_inside(first_x, first_y)) {
        unsafe { unlock_texture(texture) }?;
        return Err(e.into());
    }
    let gap = row_gap(pitch, bytes);
    unsafe { game_write(GAP_RVA, gap) };
    let black = g(BLACK_RVA);
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
        }
    }
    // eax at the unlock: the texture, as the lock left it.
    unsafe { unlock_texture(texture) }?;
    Ok(())
}
