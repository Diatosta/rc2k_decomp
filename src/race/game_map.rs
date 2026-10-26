//! `race::loadlevel_init`'s "loadgamemap" (IDA, 2026-10-16): the in-race map, 128 x 128
//! tiles of 16 x 16 pixels, only the tiles the course touches kept. `g_mapTiles` (0x4000
//! dwords) points each kept tile at its pixels in `g_mapTilePixels` (0 for none), in the
//! pixel format of entry 3 of `off_5ABAC0` (`g_mapPixelBytes` a pixel); `g_mapBlack` that
//! format's opaque black.
//!
//! The game reads `files\maps\mapxx.sq` ("SQSM", version 0x50000, the tile count, then a
//! word a tile, 0 or the tile's number from 1, then each tile's pixels as 24-bit
//! components). With `g_usePackedMaps` 0, or `srcmaps\mapxx.pcx` there, it builds the
//! same from the source pictures: the map key (`srcmaps\keyxx.pcx`, a 2048 x 2048 sheet
//! that must be 8-bit) marks the tiles within three of a used one (a 4 x 16 block's four
//! dwords not all 0), and the map picture gives their pixels.

use crate::errors::{BadInput, FatalError, Refused};
use crate::files::scratch::SCRATCH;
use crate::gfx::pcx::PCX;
use crate::gfx::picture_store::PICTURE;
use crate::gfx::pixel_format::PixelFormat;
use crate::memory::MemoryList;
use crate::pointers::GamePtr;
use crate::race::trap::file_read;
use crate::session::{Bytes, Loose};

const MAGIC: u32 = 0x4D53_5153; // "SQSM"
const VERSION: u32 = 0x5_0000;

/// IDA's address of the map's variables.
pub const START: usize = 0x5BE90D;
/// Their length.
pub const LEN: usize = 0x5BE9BD - START;

/// The in-race map's variables. The dwords are at addresses that are not multiples of 4
/// (`Loose`), as are the lists (16 bytes each, a `MemoryList`).
#[repr(C)]
pub struct MapVars {
    /// `g_mapList` (`0x5BE90D`): the list the tiles' memory is on.
    pub list: Bytes<16>,
    /// `0x5BE91D` .. `0x5BE94D`: nothing reads or writes them (a gap).
    pub _gap_10: Bytes<0x30>,
    /// `g_mapKeyList` (`0x5BE94D`): the list the key sheet's memory is on.
    pub key_list: Bytes<16>,
    /// `0x5BE95D` .. `0x5BE98D`: nothing reads or writes them (a gap).
    pub _gap_50: Bytes<0x30>,
    /// `g_mapKeyPixels` (`0x5BE98D`): the sheet, 16 MB.
    pub key_pixels: Loose,
    /// `g_mapTiles` (`0x5BE991`): each tile's place in the pixels (128 x 128 dwords).
    pub tiles: Loose,
    /// `g_mapTilePixels` (`0x5BE995`).
    pub tile_pixels: Loose,
    /// `g_mapTileCount` (`0x5BE999`).
    pub tile_count: Loose,
    /// `g_mapPicturePath` (`0x5BE99D`).
    pub picture_path: Loose,
    /// `g_mapKeyPath` (`0x5BE9A1`).
    pub key_path: Loose,
    /// `g_mapPixelBytes` (`0x5BE9A5`): bytes a pixel.
    pub pixel_bytes: Loose,
    /// `g_mapPixelFormat` (`0x5BE9A9`): the pixel format's place.
    pub format: Loose,
    /// `dword_5BE9AD`: the surface's gap after a row (`blit_map_tiles`).
    pub row_gap: Loose,
    /// `g_mapBlack` (`0x5BE9B1`): the format's opaque black.
    pub black: Loose,
    /// `g_usePackedMaps` (`0x5BE9B5`).
    pub use_packed: Loose,
    /// `dword_5BE9B9`.
    pub packed_2: Loose,
}

/// The map's variables; the first values are the image's.
pub static MAP: MapVars = MapVars {
    list: Bytes::new([0; 16]),
    _gap_10: Bytes::new([0; 0x30]),
    key_list: Bytes::new([0; 16]),
    _gap_50: Bytes::new([0; 0x30]),
    key_pixels: Loose::new(0),
    tiles: Loose::new(0),
    tile_pixels: Loose::new(0),
    tile_count: Loose::new(0),
    picture_path: Loose::new(0),
    key_path: Loose::new(0),
    pixel_bytes: Loose::new(0),
    format: Loose::new(0),
    row_gap: Loose::new(0),
    black: Loose::new(0),
    use_packed: Loose::new(1),
    packed_2: Loose::new(1),
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x5BE90D).
pub fn home() -> usize {
    crate::pointers::game_address(&MAP as *const MapVars) as usize
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

fn get(at: u32) -> Result<u32, BadInput> {
    GamePtr::new(at).read("a map word")
}

fn list(bytes: &Bytes<16>) -> *mut MemoryList {
    bytes.as_ptr().cast()
}

/// Entry 3 of the pixel kinds: (bytes a pixel, the format), into the map's variables.
fn take_format() -> Result<(u32, PixelFormat), BadInput> {
    let kind: u32 = crate::gfx::texture_upload::TEXTURE_TABLES.kinds[3].get();
    let bytes = get(kind + 4)?;
    MAP.pixel_bytes.set(bytes);
    MAP.format.set(kind + 8);
    let format = GamePtr::new(kind + 8).read::<PixelFormat>("the map's pixel format")?;
    Ok((bytes, format))
}

/// A pixel of 24-bit components (the dword read: its top byte replaced) added to the bytes
/// going into the tiles.
fn store_pixel(out: &mut Vec<u8>, bytes: u32, format: &PixelFormat, components: u32) {
    let pixel = crate::gfx::components_to_pixel::components_to_pixel(format, components | OPAQUE);
    if bytes == 2 {
        out.extend_from_slice(&(pixel as u16).to_le_bytes());
    } else {
        out.extend_from_slice(&pixel.to_le_bytes());
    }
}

/// The dword at `at` in bytes read from the game's memory (the 4th byte may be the next
/// pixel's, as in the original).
fn dword_in(data: &[u8], at: usize) -> Result<u32, BadInput> {
    data.get(at..at + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or(BadInput("a map read past the bytes it was given"))
}

fn allocate(r: &Bytes<16>, bytes: u32) -> Result<u32, Refused> {
    Ok(crate::pointers::game_address({
        crate::memory::allocate::allocate(list(r), bytes)
    }?))
}

/// `race::free_game_map` (0x41B907): both map lists freed, the sheet, tiles and pixels 0.
/// Every register preserved.
pub fn free_game_map() -> Result<(), Refused> {
    crate::memory::free_list::free_list(list(&MAP.key_list))?;
    crate::memory::free_list::free_list(list(&MAP.list))?;
    for cell in [&MAP.key_pixels, &MAP.tiles, &MAP.tile_pixels] {
        cell.set(0);
    }
    Ok(())
}

/// `race::load_packed_game_map` (0x41B785): the map from a packed file (eax the path; see
/// the header). `g_mapBlack` and `g_mapTiles` (0x10000 bytes) first; a file that does not
/// open, read, say "SQSM" or have version 0x50000 leaves the tiles as allocated and the
/// count 0. Each read of the pixels is of 0x40 tiles at most. Every register preserved.
/// Refuses a count of 0 or past 0xFFFF and a short read not of whole pixels (the original
/// counts round).
pub fn load_packed_game_map(path: u32) -> Result<(), Refused> {
    use crate::files::handle::{close, open_by_mode};
    let (bytes, format) = take_format()?;
    MAP.black
        .set(crate::gfx::components_to_pixel::components_to_pixel(
            &format, OPAQUE,
        ));
    let tiles = allocate(&MAP.list, 0x1_0000)?;
    MAP.tiles.set(tiles);
    MAP.tile_count.set(0u32);
    let Some(file) = open_by_mode(crate::pointers::host(path) as *const u8, 0)? else {
        return Ok(());
    };
    let buffer = SCRATCH.buffer.addr();
    let result = (|| -> Result<(), Refused> {
        let (_, ok) = file_read(file, buffer, 0x10, "the map's scratch buffer")?;
        let head = GamePtr::new(buffer).read_bytes(0x10, "the map's scratch buffer")?;
        if !ok || dword_in(&head, 0)? != MAGIC || dword_in(&head, 8)? != VERSION {
            return Ok(());
        }
        let count = dword_in(&head, 0xC)?;
        MAP.tile_count.set(count);
        if count == 0 || count > MOST_TILES {
            return Err(BadInput("a packed map of no tiles or more than 0xFFFF").into());
        }
        let pixels = allocate(&MAP.list, (count << 8).wrapping_mul(bytes))?;
        MAP.tile_pixels.set(pixels);
        let (_, ok) = file_read(file, buffer, 0x8000, "the map's scratch buffer")?;
        if !ok {
            return Ok(());
        }
        let words = GamePtr::new(buffer).read_bytes(2 * TILES as usize, "the map's tile words")?;
        let mut table = Vec::with_capacity(4 * TILES as usize);
        for n in 0..TILES as usize {
            let word = u16::from_le_bytes([words[2 * n], words[2 * n + 1]]) as u32;
            let at = if word == 0 {
                0
            } else {
                ((word - 1) << 8).wrapping_mul(bytes).wrapping_add(pixels)
            };
            table.extend_from_slice(&at.to_le_bytes());
        }
        GamePtr::new(tiles).write_bytes(&table, "the map's tile table")?;
        let mut out = pixels;
        let mut left = count;
        while left != 0 {
            let n = left.min(0x40);
            left -= n;
            let (got, ok) = file_read(file, buffer, n * 0x300, "the map's scratch buffer")?;
            if !ok {
                return Ok(());
            }
            if got == 0 || got % 3 != 0 {
                return Err(BadInput("a packed map's short read, not whole pixels").into());
            }
            let data = GamePtr::new(buffer).read_bytes(got as usize + 1, "the map's pixels")?;
            let mut chunk = Vec::new();
            for k in 0..got / 3 {
                store_pixel(&mut chunk, bytes, &format, dword_in(&data, 3 * k as usize)?);
            }
            GamePtr::new(out).write_bytes(&chunk, "the map's tile pixels")?;
            out += chunk.len() as u32;
        }
        Ok(())
    })();
    close(file);
    result
}

/// `race::load_game_map` (0x41B480): the map (`picture` the picture's path, `key_path` the
/// key's; see the header). The eax it leaves (`g_packedPath` when it reads the packed
/// file) is read by no caller (conversion phase 3, 2026-10-24). A key not 8-bit is fatal
/// error 0x80E48073. The key's list freed and `g_mapKeyPixels` 0 at the end; the picture
/// limits and buffer put back.
pub unsafe fn load_game_map(picture: u32, key_path: u32) -> Result<(), Refused> {
    unsafe {
        if MAP.use_packed.get() == 1
            && !crate::files::file_exists::file_exists(crate::pointers::host(picture) as *const u8)?
        {
            let path = crate::files::state::PACKED_PATH.ptr().cast::<u8>();
            let cursor = crate::strings::str_append::str_append(
                crate::pointers::host(picture) as *const u8,
                path.add(3),
            )?;
            GamePtr::new(crate::pointers::game_address(cursor) - 4)
                .write(0x0071_732Eu32, "the packed map's path")?;
            GamePtr::new(crate::pointers::game_address(path))
                .write_bytes(b"files\\", "the packed map's path")?;
            load_packed_game_map(crate::pointers::game_address(path))?;
            return Ok(());
        }
    }
    MAP.tile_count.set(0u32);
    MAP.picture_path.set(picture);
    MAP.key_path.set(key_path);
    let (bytes, format) = take_format()?;
    let tiles = allocate(&MAP.list, 0x1_0000)?;
    MAP.tiles.set(tiles);
    let key = allocate(&MAP.key_list, 0x100_0000)?;
    MAP.key_pixels.set(key);
    let kept: [u32; 3] = [
        PCX.width_limit.get(),
        PCX.height_limit.get(),
        PICTURE.buffer.get(),
    ];
    PCX.width_limit.set(KEY_SIDE);
    PCX.height_limit.set(KEY_SIDE);
    PICTURE.width.set(KEY_SIDE);
    PICTURE.height.set(KEY_SIDE);
    PICTURE.row_bytes.set(KEY_ROW_BYTES);
    PICTURE.buffer.set(key);
    let built = (|| -> Result<(), Refused> {
        if !unsafe {
            crate::gfx::read_sheet::read_sheet(
                crate::pointers::host(MAP.key_path.get()) as *const u8
            )
        }? {
            return Ok(());
        }
        if PICTURE.flags.get() & 0x100 == 0 {
            return Err(NOT_8BIT.into());
        }
        let row: u32 = PICTURE.row_bytes.get();
        // The key sheet, read in place (its extent is the allocation's).
        let sheet = GamePtr::new(key).bytes("the map key sheet")?;
        let dword = |at: u32| -> Result<u32, BadInput> {
            let at = (at.wrapping_sub(key)) as usize;
            let mut b = [0u8; 4];
            for (k, v) in b.iter_mut().enumerate() {
                *v = rc2k_core::strings::bytes::Bytes::byte(&sheet, at + k)
                    .ok_or(BadInput("a map key read past its sheet"))?;
            }
            Ok(u32::from_le_bytes(b))
        };
        // The tile table, worked on here and put back whole.
        let mut table = vec![0u32; TILES as usize];
        let stored = GamePtr::new(tiles).read_bytes(4 * TILES as usize, "the map's tile table")?;
        for (v, c) in table.iter_mut().zip(stored.as_chunks::<4>().0) {
            *v = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
        }
        let put_table = |table: &[u32]| -> Result<(), BadInput> {
            let bytes: Vec<u8> = table.iter().flat_map(|v| v.to_le_bytes()).collect();
            GamePtr::new(tiles).write_bytes(&bytes, "the map's tile table")
        };
        for ty in 0..SIDE {
            for tx in 0..SIDE {
                let mut used = 0;
                let block = key.wrapping_add(ty * 16 * row).wrapping_add(tx * 16);
                for y in 0..16 {
                    for k in 0..4 {
                        used |= dword(block.wrapping_add(y * row) + 4 * k)?;
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
                            table[(y * SIDE + x) as usize] = 1;
                        }
                    }
                }
            }
        }
        put_table(&table)?;
        let marked = table.iter().filter(|&&v| v != 0).count() as u32;
        let pixels = allocate(&MAP.list, (marked << 8).wrapping_mul(bytes))?;
        MAP.tile_pixels.set(pixels);
        if !unsafe {
            crate::gfx::read_picture_file::read_picture_file(crate::pointers::host(
                MAP.picture_path.get(),
            ))
        }? {
            return Ok(());
        }
        MAP.tile_count.set(0u32);
        let row: u32 = PICTURE.row_bytes.get();
        let mut out = pixels;
        for ty in 0..SIDE {
            for tx in 0..SIDE {
                let n = (ty * SIDE + tx) as usize;
                if table[n] == 0 {
                    continue;
                }
                table[n] = out;
                MAP.tile_count.set(MAP.tile_count.get() + 1);
                let mut from = (ty << 4)
                    .wrapping_mul(row)
                    .wrapping_add(tx << 6)
                    .wrapping_add(key);
                let mut tile = Vec::new();
                for _ in 0..16 {
                    for k in 0..16 {
                        store_pixel(&mut tile, bytes, &format, dword(from + 4 * k)?);
                    }
                    from = from.wrapping_add(row);
                }
                GamePtr::new(out).write_bytes(&tile, "the map's tile pixels")?;
                out += tile.len() as u32;
            }
        }
        put_table(&table)?;
        MAP.black
            .set(crate::gfx::components_to_pixel::components_to_pixel(
                &format, OPAQUE,
            ));
        Ok(())
    })();
    // A fatal error or a refusal leaves everything as it stood.
    built?;
    PICTURE.buffer.set(kept[2]);
    PCX.height_limit.set(kept[1]);
    PCX.width_limit.set(kept[0]);
    crate::memory::free_list::free_list(list(&MAP.key_list))?;
    MAP.key_pixels.set(0u32);
    Ok(())
}

#[cfg(test)]
mod block_tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&MAP, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(MapVars, key_list), 0x5BE94D - START);
        assert_eq!(offset_of!(MapVars, key_pixels), 0x5BE98D - START);
        assert_eq!(offset_of!(MapVars, use_packed), 0x5BE9B5 - START);
        assert_eq!(crate::moved::place(START), MAP.list.addr() as usize);
        assert_eq!(crate::moved::place(0x5BE94D), MAP.key_list.addr() as usize);
    }
}
