//! The course map's scroll in the race (from `race::scroll_maps`
//! 0x45B4E6, `race::scroll_map` 0x45B527 and `race::map_tiles` 0x45B663): the car on the
//! map (16.16 places), the map picture's corner (16-pixel steps) moved round it when it
//! nears an edge, and its place in the picture.

/// A distance on the ground as a distance on the map: n x 0x60 / 1000 (`imul`, `idiv`):
/// the quotient and the remainder.
pub fn on_map(n: u32) -> (u32, u32) {
    let v = n as i32 as i64 * 0x60;
    ((v / 1000) as i32 as u32, (v % 1000) as i32 as u32)
}

/// Whether the car's pixel, `d` from the picture's corner, is still well inside it
/// (0x34 .. 0x4B, signed).
pub fn well_inside(d: u32) -> bool {
    (d.wrapping_sub(0x34) as i32) >= 0 && (d.wrapping_sub(0x4C) as i32) < 0
}

/// The picture's corner put round the car's pixel: centred (0x40 back), in 16-pixel
/// steps.
pub fn centred_corner(pixel: u32) -> u32 {
    pixel.wrapping_add(8).wrapping_sub(0x40) & !0xF
}

/// The car's pixel on the map (its 16.16 place's whole part, signed).
pub fn pixel(place: u32) -> u32 {
    ((place as i32) >> 16) as u32
}

/// The car's place in the picture: ((place - corner << 16) - 0x300000) x 2.
pub fn picture_place(corner: u32, place: u32) -> u32 {
    (corner.wrapping_neg() << 16)
        .wrapping_add(place)
        .wrapping_sub(0x30_0000)
        << 1
}

/// The first tile of a corner (>> 4, signed).
pub fn first_tile(corner: u32) -> u32 {
    ((corner as i32) >> 4) as u32
}

/// The map picture's slot: the first plus the car's number, unless one
/// view only is shown (`session`).
pub fn picture_slot(first: u32, one_view: bool, car: u32) -> u32 {
    if one_view {
        first
    } else {
        first.wrapping_add(car)
    }
}

/// The views whose cars scroll their maps: the third view's alone (`session`), the
/// first's with one player (`views` 1), else the first's and the second's.
pub fn scrolled_views(third_only: bool, players: u32) -> &'static [usize] {
    if third_only {
        &[2]
    } else if players == 1 {
        &[0]
    } else {
        &[0, 1]
    }
}

/// The map's tiles (`race::blit_map_tiles` 0x41B931): 8 x 8 tiles
/// of 16 x 16 pixels blitted from a first tile; the tile table is 0x80 x 0x80 pointers.
pub const TILE: u32 = 16;
pub const TILES: u32 = 0x80;
pub const BLITTED: u32 = 8;

/// A tile's index in the table, None outside its 0x80 x 0x80 (unsigned: no tile, the
/// black colour drawn).
pub fn tile_index(x: u32, y: u32) -> Option<u32> {
    (x < TILES && y < TILES).then(|| y * TILES + x)
}

/// Whether any of the 8 x 8 from (x, y) is inside the table.
pub fn any_inside(x: u32, y: u32) -> bool {
    (0..BLITTED).any(|row| {
        (0..BLITTED).any(|col| tile_index(x.wrapping_add(col), y.wrapping_add(row)).is_some())
    })
}

/// `blit_map_tiles`' refusals, after the lock: a null surface, or no tile table with a
/// tile inside it.
pub fn check_tiles(pixels: u32, tiles: u32, any_inside: bool) -> Result<(), crate::Invalid> {
    if pixels == 0 {
        Err(crate::Invalid("a null surface"))
    } else if tiles == 0 && any_inside {
        Err(crate::Invalid("no map tiles"))
    } else {
        Ok(())
    }
}

/// The gap from a tile's row's end to the next row's start on the surface.
pub fn row_gap(pitch: u32, bytes: u32) -> u32 {
    pitch.wrapping_sub(bytes.wrapping_mul(TILE))
}

/// Tile (`col`, `row`)'s pixel row `r` (of the 8 x 8): the surface's row and the byte along
/// it. The original starts a tile at 16 x (`col` x `bytes` + `row` x the pitch) and steps its
/// rows by `wide` x 16 + `row_gap`: the pitch, and 16 x (`wide` - `bytes`) more when the
/// pixels are stored wider than the surface's.
pub fn tile_row(col: u32, row: u32, r: u32, bytes: u32, wide: u32) -> (u32, u32) {
    let along = (col.wrapping_mul(bytes) << 4)
        .wrapping_add(r.wrapping_mul(TILE).wrapping_mul(wide.wrapping_sub(bytes)));
    ((row << 4).wrapping_add(r), along)
}

/// Two bytes a pixel as words, any other size as dwords.
pub fn pixel_width(bytes: u32) -> u32 {
    if bytes == 2 { 2 } else { 4 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiles() {
        assert_eq!(tile_index(1, 2), Some(0x101));
        assert_eq!(tile_index(0x80, 0), None);
        assert_eq!(tile_index(0, u32::MAX), None);
        assert!(any_inside(0x7F, 0x7F) && any_inside(u32::MAX - 3, 0));
        assert!(!any_inside(0x80, 0) && !any_inside(u32::MAX - 8, 0));
        assert!(check_tiles(0, 1, false).is_err());
        assert!(check_tiles(1, 0, true).is_err());
        assert!(check_tiles(1, 0, false).is_ok());
        assert_eq!(row_gap(0x400, 2), 0x3E0);
        // The original's address: the tile's start plus r steps of wide x 16 + the gap.
        let (pitch, r) = (0x400u32, 5u32);
        for (bytes, wide) in [(2, 2), (4, 4), (3, 4)] {
            let start = (bytes << 4) + (pitch << 4);
            let original = start + r * (wide * TILE + row_gap(pitch, bytes));
            let (y, along) = tile_row(1, 1, r, bytes, wide);
            assert_eq!(y * pitch + along, original);
        }
        assert_eq!((pixel_width(2), pixel_width(3)), (2, 4));
    }

    #[test]
    fn scroll() {
        assert_eq!(on_map(1000), (0x60, 0));
        assert_eq!(
            on_map((-1001i32) as u32),
            ((-96i32) as u32, (-96i32) as u32)
        );
        assert!(well_inside(0x34) && well_inside(0x4B) && !well_inside(0x4C) && !well_inside(0x33));
        assert!(!well_inside(u32::MAX));
        assert_eq!(centred_corner(0x100), 0xC0);
        assert_eq!(centred_corner(0x107), 0xC0);
        assert_eq!(centred_corner(0x108), 0xD0);
        assert_eq!(pixel(0xFFFF_0000), u32::MAX);
        assert_eq!(
            picture_place(0x10, 0x0050_0000),
            (0x0040_0000 - 0x30_0000) << 1
        );
        assert_eq!(first_tile(0xFFFF_FFF0), u32::MAX);
        assert_eq!(picture_slot(5, false, 2), 7);
        assert_eq!(picture_slot(5, true, 2), 5);
        assert_eq!(scrolled_views(true, 2), [2]);
        assert_eq!(scrolled_views(false, 1), [0]);
        assert_eq!(scrolled_views(false, 2), [0, 1]);
    }
}
