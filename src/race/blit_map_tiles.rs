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
//! Callers: `race::map_tiles_stub`, so `race::map_tiles` (the map's scroll). Refuses (after the
//! lock, unlocking first) a null surface, and a null tile table with a tile inside the 0x80 x
//! 0x80.

use crate::errors::Refused;
use crate::gfx::screen_state::DRAW_SURFACE;
use crate::pointers::GamePtr;
use crate::race::game_map::MAP;
use rc2k_core::race::map::TILE;

/// The Rust version (see the header).
pub unsafe fn blit_map_tiles(texture: u32, first_x: u32, first_y: u32) -> Result<(), Refused> {
    use crate::d3d::{lock_texture::lock_texture, unlock_texture::unlock_texture};
    // Converted: `rc2k_core::race::map::{any_inside, check_tiles, row_gap, tile_offset,
    // tile_index, pixel_width}`.
    use rc2k_core::race::map::{
        BLITTED, any_inside, check_tiles, pixel_width, row_gap, tile_index, tile_offset,
    };
    if { lock_texture(texture) }?.carry {
        return Ok(());
    }
    let (bytes, pitch, pixels) = (
        DRAW_SURFACE.bytes_per_pixel.get(),
        DRAW_SURFACE.pitch.get(),
        DRAW_SURFACE.pixels.get(),
    );
    let tiles = MAP.tiles.get();
    if let Err(e) = check_tiles(pixels, tiles, any_inside(first_x, first_y)) {
        { unlock_texture(texture) }?;
        return Err(e.into());
    }
    let gap = row_gap(pitch, bytes);
    MAP.row_gap.set(gap);
    let black = MAP.black.get();
    for row in 0..BLITTED {
        for col in 0..BLITTED {
            let (x, y) = (first_x.wrapping_add(col), first_y.wrapping_add(row));
            let mut to = pixels.wrapping_add(tile_offset(col, row, bytes, pitch));
            let tile = match tile_index(x, y) {
                Some(n) => match GamePtr::new(tiles.wrapping_add(4 * n))
                    .read::<u32>("a map tile's pointer")
                {
                    Ok(t) => t,
                    Err(e) => {
                        { unlock_texture(texture) }?;
                        return Err(e.into());
                    }
                },
                None => 0,
            };
            let wide = pixel_width(bytes);
            // The tile's pixels, read whole once (16 x 16 of them, contiguous).
            let source = if tile != 0 {
                match GamePtr::new(tile).read_bytes(
                    (TILE * TILE) as usize * wide as usize,
                    "a map tile's pixels",
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        { unlock_texture(texture) }?;
                        return Err(e.into());
                    }
                }
            } else {
                Vec::new()
            };
            let mut at = 0usize;
            for _ in 0..TILE {
                for _ in 0..TILE {
                    // The locked texture is the platform's memory (left for its own change).
                    unsafe {
                        if wide == 2 {
                            let v = if tile != 0 {
                                u16::from_le_bytes([source[at], source[at + 1]])
                            } else {
                                black as u16
                            };
                            (crate::pointers::host(to) as *mut u16).write_unaligned(v);
                        } else {
                            let v = if tile != 0 {
                                u32::from_le_bytes([
                                    source[at],
                                    source[at + 1],
                                    source[at + 2],
                                    source[at + 3],
                                ])
                            } else {
                                black
                            };
                            (crate::pointers::host(to) as *mut u32).write_unaligned(v);
                        }
                    }
                    to = to.wrapping_add(wide);
                    at += wide as usize;
                }
                to = to.wrapping_add(gap);
            }
        }
    }
    // eax at the unlock: the texture, as the lock left it.
    { unlock_texture(texture) }?;
    Ok(())
}
