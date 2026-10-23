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
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::gfx::pixel_format::PixelFormat;
use crate::memory::MemoryList;

pub const KEY_PIXELS_RVA: usize = rva(0x5BE98D); // g_mapKeyPixels: the sheet, 16 MB
pub const TILES_RVA: usize = rva(0x5BE991); // g_mapTiles
pub const TILE_PIXELS_RVA: usize = rva(0x5BE995); // g_mapTilePixels
pub const TILE_COUNT_RVA: usize = rva(0x5BE999); // g_mapTileCount
pub const PICTURE_PATH_RVA: usize = rva(0x5BE99D); // g_mapPicturePath
pub const KEY_PATH_RVA: usize = rva(0x5BE9A1); // g_mapKeyPath
pub const PIXEL_BYTES_RVA: usize = rva(0x5BE9A5); // g_mapPixelBytes
pub const FORMAT_RVA: usize = rva(0x5BE9A9); // g_mapPixelFormat
pub const BLACK_RVA: usize = rva(0x5BE9B1); // g_mapBlack
pub const USE_PACKED_RVA: usize = rva(0x5BE9B5); // g_usePackedMaps
pub const LIST_RVA: usize = rva(0x5BE90D); // g_mapList
pub const KEY_LIST_RVA: usize = rva(0x5BE94D); // g_mapKeyList
const KINDS_RVA: usize = rva(0x5ABAC0); // off_5ABAC0: the pixel kinds
const READ_BUFFER_RVA: usize = rva(0x93E8B0); // g_pngInfo, used as the read buffer
pub const PACKED_PATH_RVA: usize = rva(0x5BE9BD); // g_packedPath
const MAGIC: u32 = 0x4D53_5153; // "SQSM"
const VERSION: u32 = 0x5_0000;
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

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

fn list(r: usize) -> *mut MemoryList {
    game_addr(r) as *mut MemoryList
}

/// Entry 3 of the pixel kinds: (bytes a pixel, the format), into the map's variables.
unsafe fn take_format() -> (u32, &'static PixelFormat) {
    unsafe {
        let kind: u32 = game_read(KINDS_RVA + 0xC);
        let bytes = get(kind + 4);
        game_write(PIXEL_BYTES_RVA, bytes);
        game_write(FORMAT_RVA, kind + 8);
        (bytes, &*((kind + 8) as *const PixelFormat))
    }
}

/// A pixel of 24-bit components (the dword read: its top byte replaced) into the tiles.
unsafe fn store_pixel(at: &mut u32, bytes: u32, format: &PixelFormat, components: u32) {
    unsafe {
        let pixel =
            crate::gfx::components_to_pixel::components_to_pixel(format, components | OPAQUE);
        if bytes == 2 {
            (*at as *mut u16).write_unaligned(pixel as u16);
            *at += 2;
        } else {
            put(*at, pixel);
            *at += 4;
        }
    }
}

fn allocate(r: usize, bytes: u32) -> Result<u32, Refused> {
    Ok(unsafe { crate::memory::allocate::allocate(list(r), bytes) }? as u32)
}

/// `race::free_game_map` (0x41B907): both map lists freed, the sheet, tiles and pixels 0.
/// Every register preserved.
pub unsafe fn free_game_map() -> Result<(), Refused> {
    unsafe {
        crate::memory::free_list::free_list(list(KEY_LIST_RVA))?;
        crate::memory::free_list::free_list(list(LIST_RVA))?;
        for r in [KEY_PIXELS_RVA, TILES_RVA, TILE_PIXELS_RVA] {
            game_write(r, 0u32);
        }
    }
    Ok(())
}

/// `race::load_packed_game_map` (0x41B785): the map from a packed file (eax the path; see
/// the header). `g_mapBlack` and `g_mapTiles` (0x10000 bytes) first; a file that does not
/// open, read, say "SQSM" or have version 0x50000 leaves the tiles as allocated and the
/// count 0. Each read of the pixels is of 0x40 tiles at most. Every register preserved.
/// Refuses a count of 0 or past 0xFFFF and a short read not of whole pixels (the original
/// counts round).
pub unsafe fn load_packed_game_map(path: u32) -> Result<(), Refused> {
    use crate::files::handle::{close, open_by_mode, read};
    unsafe {
        let (bytes, format) = take_format();
        game_write(
            BLACK_RVA,
            crate::gfx::components_to_pixel::components_to_pixel(format, OPAQUE),
        );
        let tiles = allocate(LIST_RVA, 0x1_0000)?;
        game_write(TILES_RVA, tiles);
        game_write(TILE_COUNT_RVA, 0u32);
        let Some(file) = open_by_mode(path as *const u8, 0)? else {
            return Ok(());
        };
        let buffer = game_addr(READ_BUFFER_RVA) as u32;
        let result = (|| -> Result<(), Refused> {
            let (_, ok) = read(file, buffer as *mut u8, 0x10);
            if !ok || get(buffer) != MAGIC || get(buffer + 8) != VERSION {
                return Ok(());
            }
            let count = get(buffer + 0xC);
            game_write(TILE_COUNT_RVA, count);
            if count == 0 || count > MOST_TILES {
                return Err(BadInput("a packed map of no tiles or more than 0xFFFF").into());
            }
            let pixels = allocate(LIST_RVA, (count << 8).wrapping_mul(bytes))?;
            game_write(TILE_PIXELS_RVA, pixels);
            let (_, ok) = read(file, buffer as *mut u8, 0x8000);
            if !ok {
                return Ok(());
            }
            for n in 0..TILES {
                let word = *((buffer + 2 * n) as *const u16) as u32;
                let at = if word == 0 {
                    0
                } else {
                    ((word - 1) << 8).wrapping_mul(bytes).wrapping_add(pixels)
                };
                put(tiles + 4 * n, at);
            }
            let mut out = pixels;
            let mut left = count;
            while left != 0 {
                let n = left.min(0x40);
                left -= n;
                let (got, ok) = read(file, buffer as *mut u8, n * 0x300);
                if !ok {
                    return Ok(());
                }
                if got == 0 || got % 3 != 0 {
                    return Err(BadInput("a packed map's short read, not whole pixels").into());
                }
                for k in 0..got / 3 {
                    store_pixel(&mut out, bytes, format, get(buffer + 3 * k));
                }
            }
            Ok(())
        })();
        close(file);
        result
    }
}

/// `race::load_game_map` (0x41B480): the map (eax the picture's path, ebx the key's; see
/// the header). Leaves eax `g_packedPath` when it reads the packed file, else every
/// register preserved. A key not 8-bit is fatal error 0x80E48073. The key's list freed
/// and `g_mapKeyPixels` 0 at the end; the picture limits and buffer put back.
pub unsafe fn load_game_map(entry: &Regs) -> Result<Regs, Refused> {
    use crate::gfx::pcx::{HEIGHT_LIMIT_RVA, WIDTH_LIMIT_RVA};
    use crate::gfx::picture::{FLAGS_RVA, HEIGHT_RVA, PICTURE_RVA, ROW_BYTES_RVA, WIDTH_RVA};
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(USE_PACKED_RVA) == 1 {
            let mut r = g;
            crate::race::game_map::game_part(0, &mut r)?;
            if r.carry() {
                let path = game_addr(PACKED_PATH_RVA) as *mut u8;
                let cursor =
                    crate::strings::str_append::str_append(g.eax as *const u8, path.add(3));
                put(cursor as u32 - 4, 0x0071_732E);
                (path as *mut [u8; 6]).write_unaligned(*b"files\\");
                load_packed_game_map(path as u32)?;
                g.eax = path as u32;
                return Ok(g);
            }
        }
        game_write(TILE_COUNT_RVA, 0u32);
        game_write(PICTURE_PATH_RVA, g.eax);
        game_write(KEY_PATH_RVA, g.ebx);
        let (bytes, format) = take_format();
        let tiles = allocate(LIST_RVA, 0x1_0000)?;
        game_write(TILES_RVA, tiles);
        let key = allocate(KEY_LIST_RVA, 0x100_0000)?;
        game_write(KEY_PIXELS_RVA, key);
        let kept: [u32; 3] = [
            game_read(WIDTH_LIMIT_RVA),
            game_read(HEIGHT_LIMIT_RVA),
            game_read(PICTURE_RVA),
        ];
        game_write(WIDTH_LIMIT_RVA, KEY_SIDE);
        game_write(HEIGHT_LIMIT_RVA, KEY_SIDE);
        game_write(WIDTH_RVA, KEY_SIDE);
        game_write(HEIGHT_RVA, KEY_SIDE);
        game_write(ROW_BYTES_RVA, KEY_ROW_BYTES);
        game_write(PICTURE_RVA, key);
        let built = (|| -> Result<(), Refused> {
            let mut r = Regs {
                eax: game_read(KEY_PATH_RVA),
                ..g
            };
            crate::race::game_map::game_part(1, &mut r)?;
            if r.carry() {
                return Ok(());
            }
            if game_read::<u32>(FLAGS_RVA) & 0x100 == 0 {
                return Err(NOT_8BIT.into());
            }
            let row: u32 = game_read(ROW_BYTES_RVA);
            for ty in 0..SIDE {
                for tx in 0..SIDE {
                    let mut used = 0;
                    let block = key.wrapping_add(ty * 16 * row).wrapping_add(tx * 16);
                    for y in 0..16 {
                        for k in 0..4 {
                            used |= get(block.wrapping_add(y * row) + 4 * k);
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
                                put(tiles + y * 0x200 + 4 * x, 1);
                            }
                        }
                    }
                }
            }
            let marked = (0..TILES).filter(|&n| get(tiles + 4 * n) != 0).count() as u32;
            let pixels = allocate(LIST_RVA, (marked << 8).wrapping_mul(bytes))?;
            game_write(TILE_PIXELS_RVA, pixels);
            let mut r = Regs {
                eax: game_read(PICTURE_PATH_RVA),
                ..g
            };
            crate::race::game_map::game_part(2, &mut r)?;
            if r.carry() {
                return Ok(());
            }
            game_write(TILE_COUNT_RVA, 0u32);
            let row: u32 = game_read(ROW_BYTES_RVA);
            let mut out = pixels;
            for ty in 0..SIDE {
                for tx in 0..SIDE {
                    let at = tiles + 4 * (ty * SIDE + tx);
                    if get(at) == 0 {
                        continue;
                    }
                    put(at, out);
                    game_write(TILE_COUNT_RVA, game_read::<u32>(TILE_COUNT_RVA) + 1);
                    let mut from = (ty << 4)
                        .wrapping_mul(row)
                        .wrapping_add(tx << 6)
                        .wrapping_add(key);
                    for _ in 0..16 {
                        for k in 0..16 {
                            store_pixel(&mut out, bytes, format, get(from + 4 * k));
                        }
                        from = from.wrapping_add(row);
                    }
                }
            }
            game_write(
                BLACK_RVA,
                crate::gfx::components_to_pixel::components_to_pixel(format, OPAQUE),
            );
            Ok(())
        })();
        // A fatal error or a refusal leaves everything as it stood.
        built?;
        game_write(PICTURE_RVA, kept[2]);
        game_write(HEIGHT_LIMIT_RVA, kept[1]);
        game_write(WIDTH_LIMIT_RVA, kept[0]);
        crate::memory::free_list::free_list(list(KEY_LIST_RVA))?;
        game_write(KEY_PIXELS_RVA, 0u32);
    }
    Ok(g)
}

/// `load_game_map`'s calls in the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let ok = match place {
            0 => crate::files::file_exists::file_exists(r.eax as *const u8)?,
            1 => crate::gfx::read_sheet::read_sheet(r.eax as *const u8)?,
            _ => crate::gfx::read_picture_file::read_picture_file(r.eax as *mut u8)?,
        };
        r.set_carry(!ok);
    }
    Ok(())
}

// ---- for the self-tests ----
