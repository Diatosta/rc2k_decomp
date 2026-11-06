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

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::gfx::owner::gfx;
use crate::race::game_map::{GAME_MAP, Tile};
use crate::race::owner_frame::Frame;
use rc2k_core::race::map::TILE;

/// The Rust version (see the header).
pub fn blit_map_tiles(
    frame: &Frame,
    texture: u32,
    first_x: u32,
    first_y: u32,
) -> Result<(), Refused> {
    let drawing = &gfx().drawing;
    use crate::d3d::{lock_texture::lock_texture, unlock_texture::unlock_texture};
    // Converted: `rc2k_core::race::map::{any_inside, check_tiles, row_gap, tile_row,
    // tile_index, pixel_width}`.
    use rc2k_core::race::map::{
        BLITTED, any_inside, check_tiles, pixel_width, row_gap, tile_index, tile_row,
    };
    if { lock_texture(texture) }?.carry {
        return Ok(());
    }
    let (bytes, pitch, pixels) = (
        drawing.draw_surface.bytes_per_pixel.get(),
        drawing.draw_surface.pitch.get(),
        crate::gfx::surface::address()?,
    );
    let tiles = u32::from(GAME_MAP.is_taken());
    if let Err(e) = check_tiles(pixels, tiles, any_inside(first_x, first_y)) {
        { unlock_texture(texture) }?;
        return Err(e.into());
    }
    frame.map.row_gap.set(row_gap(pitch, bytes));
    let surface = crate::gfx::rows::Rows::new(pixels, pitch, drawing.draw_surface.height.get());
    let black = frame.map.black.get();
    for row in 0..BLITTED {
        for col in 0..BLITTED {
            let (x, y) = (first_x.wrapping_add(col), first_y.wrapping_add(row));
            let wide = pixel_width(bytes);
            // The tile's pixels, copied out whole once (16 x 16 of them, contiguous); none
            // drawn black. A tile marked but never filled holds 1 in the original's table:
            // read there, refused.
            let len = (TILE * TILE) as usize * wide as usize;
            let source = match tile_index(x, y) {
                Some(n) => GAME_MAP.with(|m| match m.tiles.get(n as usize) {
                    Some(Tile::None) => Ok(None),
                    Some(Tile::At(at)) => m
                        .pixels
                        .get(*at..*at + len)
                        .map(|p| Some(p.to_vec()))
                        .ok_or(BadInput("a map tile's pixels past their block")),
                    Some(Tile::Marked) => Err(BadInput("a map tile marked but never filled")),
                    None => Err(BadInput("a map tile past the table")),
                }),
                None => Ok(Ok(None)),
            };
            let source = match source.and_then(|s| s) {
                Ok(s) => s,
                Err(e) => {
                    { unlock_texture(texture) }?;
                    return Err(e.into());
                }
            };
            let tile = u32::from(source.is_some());
            let source = source.unwrap_or_default();
            let mut at = 0usize;
            for r in 0..TILE {
                // The original steps a tile's rows by its pixels' bytes and the gap: the
                // texture's pitch, and `wide` - `bytes` more a pixel when they differ.
                let (y, along) = tile_row(col, row, r, bytes, wide);
                // The row's pixels, `wide` bytes each, written at once (the locked texture
                // is the platform's memory).
                let mut row = Vec::with_capacity(TILE as usize * wide as usize);
                for _ in 0..TILE {
                    if wide == 2 {
                        let v = if tile != 0 {
                            u16::from_le_bytes([source[at], source[at + 1]])
                        } else {
                            black as u16
                        };
                        row.extend_from_slice(&v.to_le_bytes());
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
                        row.extend_from_slice(&v.to_le_bytes());
                    }
                    at += wide as usize;
                }
                surface.write(y, along, &row, "a map texture's row")?;
            }
        }
    }
    // eax at the unlock: the texture, as the lock left it.
    { unlock_texture(texture) }?;
    Ok(())
}
