//! The in-race map: 128 x 128 tiles of 16 x 16 pixels, only the tiles the course touches kept.
//! `MapVars::tiles` (0x4000 dwords) points each kept tile at its pixels in `tile_pixels` (0 for
//! none), in the pixel format of entry 3 of `Textures.kinds[0]` (`pixel_bytes` a pixel); `black`
//! is that format's opaque black.
//!
//! The packed map is `files\maps\mapxx.sq` ("SQSM", version 0x50000, the tile count, then a word a
//! tile, 0 or the tile's number from 1, then each tile's pixels as 24-bit components). With
//! `use_packed` 0, or `srcmaps\mapxx.pcx` there, the map is built from the source pictures: the
//! map key (`srcmaps\keyxx.pcx`, a 2048 x 2048 sheet that must be 8-bit) marks the tiles within
//! three of a used one (a 4 x 16 block's four dwords not all 0), and the map picture gives their
//! pixels.

use crate::errors::{BadInput, FatalError, Refused};
use crate::gfx::pixel_format::PixelFormat;
use crate::memory::session::{PlainBytes, PlainDword};
use crate::race::owner::Race;
use crate::race::owner_frame::Frame;

const MAGIC: u32 = 0x4D53_5153; // "SQSM"
const VERSION: u32 = 0x5_0000;

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

const SIDE: u32 = 0x80;
const TILES: u32 = SIDE * SIDE;
const MOST_TILES: u32 = 0xFFFF;
const OPAQUE: u32 = 0xFF00_0000;
const KEY_SIDE: u32 = 0x800;
const KEY_ROW_BYTES: u32 = 0x2000;
const NOT_8BIT: FatalError = FatalError {
    code: 0x80E4_8073,
    message: "Game mapkey file must be 8bit",
};

/// Entry 3 of the pixel kinds: (bytes a pixel, the format's handle, the format).
pub fn take_format(gfx: &crate::gfx::owner::Gfx) -> Result<(u32, u32, PixelFormat), BadInput> {
    let card = &gfx.card;
    let kind = card
        .texture_tables
        .kind_index(3)
        .ok_or(BadInput("the map's pixel kind has no description"))?;
    let description = card.texture_tables.descriptions[kind].get();
    let handle = crate::gfx::pixel_format::texture_kind_format(3, gfx);
    Ok((description.bytes_per_pixel, handle, description.format))
}

/// Adds a pixel of 24-bit components (the dword read, its top byte replaced) to the bytes going
/// into the tiles.
fn store_pixel(
    out: &mut Vec<u8>,
    bytes: u32,
    format: &PixelFormat,
    components: u32,
    gfx: &crate::gfx::owner::Gfx,
) {
    let pixel =
        crate::gfx::components_to_pixel::components_to_pixel(format, components | OPAQUE, gfx);
    if bytes == 2 {
        out.extend_from_slice(&(pixel as u16).to_le_bytes());
    } else {
        out.extend_from_slice(&pixel.to_le_bytes());
    }
}

/// The dword at `at` in bytes read from the game's memory (its 4th byte may be the next pixel's).
fn dword_in(data: &[u8], at: usize) -> Result<u32, BadInput> {
    data.get(at..at + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or(BadInput("a map read past the bytes it was given"))
}

/// A tile of the map: none (drawn black), marked as used but not filled, or its pixels' byte
/// offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    None,
    Marked,
    At(usize),
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
    fn empty(pixel_bytes: u32, format: u32) -> Self {
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

/// Stores a loaded map: its variables, the tile table (0x10000 bytes counted on the map's list)
/// and the pixels' block (counted on the list too).
pub fn store_game_map(race: &Race, loaded: LoadedMap) -> Result<(), Refused> {
    let vars = race.frame.map();
    vars.pixel_bytes.set(loaded.pixel_bytes);
    vars.format.set(loaded.format);
    if let Some(black) = loaded.black {
        vars.black.set(black);
    }
    let LoadedMap {
        map: GameMap { tiles, pixels },
        tile_count,
        ..
    } = loaded;
    race.course.stage.game_map.take(
        &vars.list,
        0x1_0000,
        GameMap {
            tiles,
            pixels: Vec::new(),
        },
    )?;
    vars.tile_count.set(tile_count);
    crate::memory::owned::count_on(&vars.list, pixels.len() as u32)?;
    race.course.stage.game_map.with(|m| m.pixels = pixels)?;
    Ok(())
}

/// `bytes` put into the pixels' block at byte `at`; refused past it.
fn put_pixels(block: &mut [u8], at: usize, bytes: &[u8]) -> Result<(), BadInput> {
    block
        .get_mut(at..at + bytes.len())
        .map(|p| p.copy_from_slice(bytes))
        .ok_or(BadInput("the map's tile pixels past their block"))
}

/// Frees both map lists and zeroes the sheet, tiles and pixels.
pub fn free_game_map(race: &Race, gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    let frame = &race.frame;
    crate::gfx::picture::give_back(&gfx.pictures.buffers, crate::gfx::picture::Buffer::Key);
    crate::memory::free_list::free_list(&frame.map().key_list)?;
    crate::memory::free_list::free_list(&frame.map().list)?;
    race.course.stage.game_map.give_back();
    for cell in [
        &frame.map().key_pixels,
        &frame.map().tiles,
        &frame.map().tile_pixels,
    ] {
        cell.set(0);
    }
    Ok(())
}

/// Reads the map from a packed file. A file that does not open, read, say "SQSM" or have version
/// 0x50000 gives no tiles and the count 0. Each read of the pixels is of 0x40 tiles at most.
/// Refuses a count of 0 or past 0xFFFF and a short read not of whole pixels. `path` is without
/// its NUL.
pub fn load_packed_game_map(
    path: &[u8],
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<LoadedMap, Refused> {
    let work = &system.work();
    use crate::files::handle::{close, open_by_mode};
    let (bytes, handle, format) = take_format(gfx)?;
    let mut loaded = LoadedMap::empty(bytes, handle);
    loaded.black = Some(crate::gfx::components_to_pixel::components_to_pixel(
        &format, OPAQUE, gfx,
    ));
    let Some(file) = open_by_mode(path, 0)? else {
        return Ok(loaded);
    };
    let result = (|| -> Result<(), Refused> {
        let (_, ok) = work
            .files
            .read_from(file, 0x10, "the map's scratch buffer")?;
        let head = work
            .files
            .buffer
            .read_at(0, 0x10, "the map's scratch buffer")?;
        if !ok || dword_in(&head, 0)? != MAGIC || dword_in(&head, 8)? != VERSION {
            return Ok(());
        }
        let count = dword_in(&head, 0xC)?;
        loaded.tile_count = count;
        if count == 0 || count > MOST_TILES {
            return Err(BadInput("a packed map of no tiles or more than 0xFFFF").into());
        }
        loaded.map.pixels = vec![0; (count << 8).wrapping_mul(bytes) as usize];
        let (_, ok) = work
            .files
            .read_from(file, 0x8000, "the map's scratch buffer")?;
        if !ok {
            return Ok(());
        }
        let words = work
            .files
            .buffer
            .read_at(0, 2 * TILES as usize, "the map's tile words")?;
        for (n, tile) in loaded.map.tiles.iter_mut().enumerate() {
            let word = u16::from_le_bytes([words[2 * n], words[2 * n + 1]]) as u32;
            *tile = if word == 0 {
                Tile::None
            } else {
                Tile::At(((word - 1) << 8).wrapping_mul(bytes) as usize)
            };
        }
        let mut out = 0usize;
        let mut left = count;
        while left != 0 {
            let n = left.min(0x40);
            left -= n;
            let (got, ok) = work
                .files
                .read_from(file, n * 0x300, "the map's scratch buffer")?;
            if !ok {
                return Ok(());
            }
            if got == 0 || got % 3 != 0 {
                return Err(BadInput("a packed map's short read, not whole pixels").into());
            }
            let data = work
                .files
                .buffer
                .read_at(0, got as usize + 1, "the map's pixels")?;
            let mut chunk = Vec::new();
            for k in 0..got / 3 {
                store_pixel(
                    &mut chunk,
                    bytes,
                    &format,
                    dword_in(&data, 3 * k as usize)?,
                    gfx,
                );
            }
            put_pixels(&mut loaded.map.pixels, out, &chunk)?;
            out += chunk.len();
        }
        Ok(())
    })();
    close(file);
    result?;
    Ok(loaded)
}

/// Reads the map: from the packed file when `use_packed` is 1 and the source picture is not
/// there, else from the key sheet and the picture. A key that is not 8-bit is fatal error
/// 0x80E48073. At the end the key's list is freed and `key_pixels` is 0; the picture limits and
/// buffer are put back.
pub fn load_game_map(
    frame: &Frame,
    picture: &dyn crate::memory::session::TextPlace,
    key_path: &dyn crate::memory::session::TextPlace,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<LoadedMap, Refused> {
    let pictures = &gfx.pictures;
    if frame.map().use_packed.get() == 1
        && !crate::files::file_exists::file_exists(&picture.text("a map picture's path")?)?
    {
        // The packed path: "files\" at the start, the picture's path after 3 bytes with its last
        // four bytes made ".sq" and a NUL.
        let path = &crate::memory::session::Bytes::<{ crate::files::state::PACKED_PATH_LEN }>::new(
            [0; crate::files::state::PACKED_PATH_LEN],
        );
        let text = picture.text("a text to append")?;
        let len = text.len() as u32;
        path.write_at(3, &text, "a text to append")?;
        // At `3 + len - 4`: before the buffer when the path is empty, so refused.
        let tail = (3 + len as usize).checked_sub(4);
        path.write_at(
            tail.ok_or(crate::errors::BadInput("past the end of a buffer"))?,
            &0x0071_732Eu32.to_le_bytes(),
            "the packed map's path",
        )?;
        path.write_at(0, b"files\\", "the packed map's path")?;
        // The packed path is built in the buffer.
        return load_packed_game_map(
            &crate::files::cstring(path, "the packed map's path")?,
            system,
            gfx,
        );
    }
    let (bytes, handle, format) = take_format(gfx)?;
    let mut loaded = LoadedMap::empty(bytes, handle);
    // The key sheet is a picture buffer (Rust's, `gfx::picture`), counted on its list; `key_pixels`
    // holds its number.
    let key = crate::gfx::picture::take(
        &gfx.pictures.buffers,
        crate::gfx::picture::Buffer::Key,
        &frame.map().key_list,
        0x100_0000,
    )?;
    frame.map().key_pixels.set(key);
    let kept: [u32; 3] = [
        pictures.pcx.width_limit.get(),
        pictures.pcx.height_limit.get(),
        pictures.picture.buffer.get(),
    ];
    pictures.pcx.width_limit.set(KEY_SIDE);
    pictures.pcx.height_limit.set(KEY_SIDE);
    pictures.picture.width.set(KEY_SIDE);
    pictures.picture.height.set(KEY_SIDE);
    pictures.picture.row_bytes.set(KEY_ROW_BYTES);
    pictures.picture.buffer.set(key);
    let built = (|| -> Result<(), Refused> {
        if !crate::gfx::read_sheet::read_sheet(&key_path.text("the key sheet's path")?, gfx)? {
            return Ok(());
        }
        if pictures.picture.flags.get() & 0x100 == 0 {
            return Err(NOT_8BIT.into());
        }
        let row: u32 = pictures.picture.row_bytes.get();
        // The key sheet, read in its buffer (offsets from its start).
        let dword = |sheet: &[u8], at: u32| -> Result<u32, BadInput> {
            sheet
                .get(at as usize..at as usize + 4)
                .map(|b| u32::from_le_bytes(b.try_into().expect("four bytes")))
                .ok_or(BadInput("a map key read past its sheet"))
        };
        let table = &mut loaded.map.tiles;
        crate::gfx::picture::with_buffer(
            &gfx.pictures.buffers,
            key,
            |sheet| -> Result<(), BadInput> {
                for ty in 0..SIDE {
                    for tx in 0..SIDE {
                        let mut used = 0;
                        let block = (ty * 16 * row).wrapping_add(tx * 16);
                        for y in 0..16 {
                            for k in 0..4 {
                                used |= dword(sheet, block.wrapping_add(y * row) + 4 * k)?;
                            }
                        }
                        if used == 0 {
                            continue;
                        }
                        for dy in -3i32..=3 {
                            let y = (dy + ty as i32) as u32;
                            if y >= SIDE {
                                continue;
                            }
                            for dx in -3i32..=3 {
                                let x = (dx + tx as i32) as u32;
                                if x < SIDE {
                                    table[(y * SIDE + x) as usize] = Tile::Marked;
                                }
                            }
                        }
                    }
                }
                Ok(())
            },
        )??;
        let marked = table.iter().filter(|&&v| v != Tile::None).count() as u32;
        loaded.map.pixels = vec![0; (marked << 8).wrapping_mul(bytes) as usize];
        if !crate::gfx::read_picture_file::read_picture_file(picture, gfx, system)? {
            return Ok(());
        }
        let row: u32 = pictures.picture.row_bytes.get();
        let mut tile_count = 0u32;
        // The marked tiles' pixels made from the picture (read into the key's buffer), then put in
        // the map's pixels in one go.
        let pixels = crate::gfx::picture::with_buffer(
            &gfx.pictures.buffers,
            key,
            |sheet| -> Result<Vec<u8>, BadInput> {
                let mut pixels = Vec::new();
                for ty in 0..SIDE {
                    for tx in 0..SIDE {
                        let n = (ty * SIDE + tx) as usize;
                        if table[n] == Tile::None {
                            continue;
                        }
                        table[n] = Tile::At(pixels.len());
                        tile_count += 1;
                        let mut from = (ty << 4).wrapping_mul(row).wrapping_add(tx << 6);
                        for _ in 0..16 {
                            for k in 0..16 {
                                store_pixel(
                                    &mut pixels,
                                    bytes,
                                    &format,
                                    dword(sheet, from + 4 * k)?,
                                    gfx,
                                );
                            }
                            from = from.wrapping_add(row);
                        }
                    }
                }
                Ok(pixels)
            },
        )??;
        put_pixels(&mut loaded.map.pixels, 0, &pixels)?;
        loaded.tile_count = tile_count;
        loaded.black = Some(crate::gfx::components_to_pixel::components_to_pixel(
            &format, OPAQUE, gfx,
        ));
        Ok(())
    })();
    // A fatal error or a refusal leaves everything as it stood.
    built?;
    pictures.picture.buffer.set(kept[2]);
    pictures.pcx.height_limit.set(kept[1]);
    pictures.pcx.width_limit.set(kept[0]);
    crate::gfx::picture::give_back(&gfx.pictures.buffers, crate::gfx::picture::Buffer::Key);
    crate::memory::free_list::free_list(&frame.map().key_list)?;
    frame.map().key_pixels.set(0u32);
    Ok(loaded)
}
