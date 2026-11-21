//! Draws the map's tiles into its picture: locks the given texture, then fills 8 x 8 tiles of 16 x
//! 16 pixels from the given tile on. A tile is the pixels `map_tiles` holds for it (0x80 x 0x80
//! pointers, 16 x 16 pixels each) or, for a tile without them or outside the 0x80 x 0x80, the the
//! map's black colour; the pixels are written as words on a surface of 2 bytes a pixel and as
//! dwords on any other, at the tile's place on the locked surface (`surface`). The texture is
//! unlocked at the end. Refused (after the lock, unlocking first): a null surface, and a null tile
//! table with a tile inside the 0x80 x 0x80.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::memory::owned::Owned;
use crate::race::game_map_store::GameMap;
use crate::race::game_map_store::Tile;
use crate::race::owner_frame::Frame;
use rc2k_core::race::map::TILE;

pub(super) fn blit_map_tiles(
    frame: &Frame,
    game_map: &Owned<GameMap>,
    texture: u32,
    first_x: u32,
    first_y: u32,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let drawing = &gfx.drawing;
    use crate::d3d::{lock_texture::lock_texture_in, unlock_texture::unlock_texture};
    use rc2k_core::race::map::{
        BLITTED, any_inside, check_tiles, pixel_width, tile_index, tile_row,
    };
    if !lock_texture_in(gfx, texture)? {
        return Ok(());
    }
    let (bytes, pitch, pixels) = (
        drawing.draw_surface.bytes_per_pixel.get(),
        drawing.draw_surface.pitch.get(),
        crate::gfx::surface::handle(gfx)?,
    );
    let tiles = u32::from(game_map.is_taken());
    if let Err(e) = check_tiles(pixels, tiles, any_inside(first_x, first_y)) {
        { unlock_texture(gfx, texture) }?;
        return Err(e.into());
    }
    let surface = crate::gfx::rows::Rows::new(pixels, pitch, drawing.draw_surface.height.get());
    let black = frame.map().black;
    for row in 0..BLITTED {
        for col in 0..BLITTED {
            let (x, y) = (first_x.wrapping_add(col), first_y.wrapping_add(row));
            let wide = pixel_width(bytes);
            // The tile's pixels, copied out whole once (16 x 16 of them, contiguous); none drawn
            // black. A tile marked but never filled holds 1 in the table: read there, refused.
            let len = (TILE * TILE) as usize * wide as usize;
            let source = match tile_index(x, y) {
                Some(n) => game_map.with(|m| match m.tiles.get(n as usize) {
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
                    { unlock_texture(gfx, texture) }?;
                    return Err(e.into());
                }
            };
            let tile = u32::from(source.is_some());
            let source = source.unwrap_or_default();
            let mut at = 0usize;
            for r in 0..TILE {
                // A tile's rows step by its pixels' bytes and the gap: the texture's pitch, and
                // `wide` - `bytes` more a pixel when they differ.
                let (y, along) = tile_row(col, row, r, bytes, wide);
                // The row's pixels, `wide` bytes each, written at once (the locked texture is the
                // platform's memory).
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
    // The unlock takes the texture as the lock left it.
    { unlock_texture(gfx, texture) }?;
    Ok(())
}
