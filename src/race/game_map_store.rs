//! The in-race map's variables and tiles.

pub const SIDE: u32 = 0x80;
pub const TILES: u32 = SIDE * SIDE;

/// The in-race map's variables.
pub struct MapVars {
    /// Bytes a pixel.
    pub pixel_bytes: u32,
    /// The pixel format's opaque black.
    pub black: u32,
}

impl MapVars {
    pub const fn new() -> Self {
        MapVars {
            pixel_bytes: 0,
            black: 0,
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
    /// The format's opaque black; `None` when the loader ended before it was worked out.
    pub black: Option<u32>,
}

impl LoadedMap {
    /// A map of no tiles, none of them used.
    pub(super) fn empty(pixel_bytes: u32) -> Self {
        Self {
            map: GameMap {
                tiles: vec![Tile::None; TILES as usize],
                pixels: Vec::new(),
            },
            tile_count: 0,
            pixel_bytes,
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
