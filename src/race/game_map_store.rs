//! The in-race map's variables and tiles.

use crate::memory::session::{PlainBytes, PlainDword};

pub const SIDE: u32 = 0x80;
pub const TILES: u32 = SIDE * SIDE;

/// The in-race map's variables.
#[derive(rc2k_derive::Cells)]
pub struct MapVars {
    /// The list the tiles' memory is on.
    pub list: crate::memory::ListCells,
    /// Unused (a gap).
    pub _gap_10: PlainBytes<0x30>,
    /// The list the key sheet's memory is on.
    pub key_list: crate::memory::ListCells,
    /// Unused (a gap).
    pub _gap_50: PlainBytes<0x30>,
    /// The key sheet, 16 MB.
    pub key_pixels: PlainDword,
    /// Each tile's place in the pixels (128 x 128 dwords); 0 since the map is Rust's (`StageData::game_map`).
    pub tiles: PlainDword,
    /// The pixels' place (0 as `tiles`).
    pub tile_pixels: PlainDword,
    /// The count of tiles used.
    pub tile_count: PlainDword,
    /// Not used: the paths are arguments.
    pub picture_path: PlainDword,
    /// Not used: the paths are arguments.
    pub key_path: PlainDword,
    /// Bytes a pixel.
    pub pixel_bytes: PlainDword,
    /// The pixel format, by handle (`gfx::pixel_format::layout`).
    pub format: PlainDword,
    /// The surface's gap after a row (`blit_map_tiles`).
    pub row_gap: PlainDword,
    /// The format's opaque black.
    pub black: PlainDword,
    /// 1 to read the packed map file.
    pub use_packed: PlainDword,
    /// Set to 1 outside the editor (0 in it) by `race::loadlevel`, next to `use_packed`; nothing
    /// reads it.
    pub packed_unless_editor: PlainDword,
}

/// The map's variables with their first values.
impl MapVars {
    pub const fn new() -> Self {
        MapVars {
            list: crate::memory::ListCells::empty(),
            _gap_10: PlainBytes::new([0; 0x30]),
            key_list: crate::memory::ListCells::empty(),
            _gap_50: PlainBytes::new([0; 0x30]),
            key_pixels: PlainDword::new(0),
            tiles: PlainDword::new(0),
            tile_pixels: PlainDword::new(0),
            tile_count: PlainDword::new(0),
            picture_path: PlainDword::new(0),
            key_path: PlainDword::new(0),
            pixel_bytes: PlainDword::new(0),
            format: PlainDword::new(0),
            row_gap: PlainDword::new(0),
            black: PlainDword::new(0),
            use_packed: PlainDword::new(1),
            packed_unless_editor: PlainDword::new(1),
        }
    }
}

/// The in-race map (`tiles` and `tile_pixels`, on `list`): the 128 x 128 tiles and the pixels of
/// those used, 16 x 16 a tile, `pixel_bytes` a pixel.
pub struct GameMap {
    pub tiles: Vec<Tile>,
    pub pixels: Vec<u8>,
}

/// What a loader makes: the map, and the variables that go with it.
pub struct LoadedMap {
    /// The tiles, and the pixels' block (zeroed where the file gave none; empty for none).
    pub map: GameMap,
    /// The count of tiles used.
    pub tile_count: u32,
    /// Bytes a pixel.
    pub pixel_bytes: u32,
    /// The pixel format's handle.
    pub format: u32,
    /// The format's opaque black; `None` when the loader ended before it was worked out.
    pub black: Option<u32>,
}

impl LoadedMap {
    /// A map of no tiles, none of them used.
    pub(super) fn empty(pixel_bytes: u32, format: u32) -> Self {
        Self {
            map: GameMap {
                tiles: vec![Tile::None; TILES as usize],
                pixels: Vec::new(),
            },
            tile_count: 0,
            pixel_bytes,
            format,
            black: None,
        }
    }
}

/// A tile of the map: none (drawn black), marked as used but not filled, or its pixels' byte
/// offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    None,
    Marked,
    At(usize),
}
