//! `race::blit_map_tiles` (0x41B931): the map's tiles into its picture: the texture eax
//! locked (`d3d::lock_texture`; carry: nothing), then 8 x 8 tiles of 16 x 16 pixels from
//! tile (ecx, edx) on: each the tile's pixels (`g_mapTiles`, 0x80 x 0x80 pointers, 16 x 16
//! pixels each) or, for a tile without them or outside the 0x80 x 0x80, the colour
//! `g_mapBlack`, as words on a surface of 2 bytes a pixel and as dwords on any other, at the
//! tile's place on the locked surface (`g_surface`); the gap between a tile's rows kept in
//! `dword_5BE9AD`; unlocked (`d3d::unlock_texture`, eax as the lock left it).
//!
//! In:  eax = the texture, ecx / edx = the first tile's column and row
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `race::map_tiles_stub`, so `race::map_tiles` (the map's scroll). It begins with
//! a call: a private copy (its two calls relocated) and a manual patch. The lock and the
//! unlock through `part` (0, 1; in the game through the game: the platform's, section 15).
//! Refuses (after the lock, unlocking first) a null surface, and a null tile table with a
//! tile inside the 0x80 x 0x80. Shadow mode runs the original alone (it tells the device).
//! Self-test: a copy whose two calls go to stand-ins (the lock failing or not), against the
//! Rust version, on a surface and tiles of the test's: 2 and 4 bytes a pixel, tiles inside,
//! across the edge and outside.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::mud::{copy, run_both_from};
use crate::service::harness::{Region, at, capture, restore};
use rc2k_core::race::map::{TILE, TILES};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::blit_map_tiles", rva(0x41B931), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&[0x1, 0xF6])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0xFD;
const BYTES_RVA: usize = rva(0x4F49AC); // g_surface.bytes_per_pixel
const PITCH_RVA: usize = rva(0x4F49BC); // g_surface.pitch
const PIXELS_RVA: usize = rva(0x4F49C0); // g_surface.pixels
const TILES_RVA: usize = rva(0x5BE991); // g_mapTiles
const BLACK_RVA: usize = rva(0x5BE9B1); // g_mapBlack
const GAP_RVA: usize = rva(0x5BE9AD); // dword_5BE9AD
const LOCK_RVA: usize = rva(0x41AA4B); // d3d::lock_texture
const UNLOCK_RVA: usize = rva(0x41AB02); // d3d::unlock_texture

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn blit_map_tiles(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(0, &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    // Converted: `rc2k_core::race::map::{any_inside, check_tiles, row_gap, tile_offset,
    // tile_index, pixel_width}`.
    use rc2k_core::race::map::{
        BLITTED, any_inside, check_tiles, pixel_width, row_gap, tile_index, tile_offset,
    };
    let (bytes, pitch, pixels) = (g(BYTES_RVA), g(PITCH_RVA), g(PIXELS_RVA));
    let tiles = g(TILES_RVA);
    let (first_x, first_y) = (entry.ecx, entry.edx);
    if let Err(e) = check_tiles(pixels, tiles, any_inside(first_x, first_y)) {
        let mut u = r;
        part(1, &mut u)?;
        return Err(e.into());
    }
    let gap = row_gap(pitch, bytes);
    unsafe { game_write(GAP_RVA, gap) };
    let black = g(BLACK_RVA);
    // The registers the loops leave for the unlock: esi and edi where the last tile left
    // them, ebp the gap, ebx the rows done, ecx back at the first column, edx past the rows.
    let (mut last_from, mut last_to) = (0u32, 0u32);
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
            (last_from, last_to) = (if tile != 0 { from } else { 0 }, to);
        }
    }
    r.esi = last_from;
    r.edi = last_to;
    r.ebp = gap;
    r.ebx = BLITTED;
    r.ecx = first_x;
    r.edx = first_y.wrapping_add(BLITTED);
    part(1, &mut r)?;
    Ok(*entry)
}

/// In the game: the platform's lock and unlock through the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::calls::call(if place == 0 { LOCK_RVA } else { UNLOCK_RVA }, r);
    Ok(())
}

/// The Rust version with the game's calls: for Rust callers.
pub unsafe fn blit_map_tiles_in_game(r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { blit_map_tiles(r, &mut game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { blit_map_tiles(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The test's lock fails (carry) while this is set.
static LOCK_FAILS: AtomicBool = AtomicBool::new(false);

fn note(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    if place == 0 {
        r.set_carry(LOCK_FAILS.load(Ordering::SeqCst));
        r.eax = 0x5107;
    }
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call d3d::lock_texture
    let rel = (0x41AA4Bi64 - (0x41B932 + 5)) as i32;
    let b = rel.to_le_bytes();
    let copy = match copy(
        0x41B931,
        LEN,
        &[0x60, 0xE8, b[0], b[1], b[2], b[3]],
        &[0x1, 0xF6],
        &[(0x41AA4B, 0), (0x41AB02, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note);
    let kept: Vec<Region> = vec![
        (game_addr(BYTES_RVA) as u32, 4),
        at(PITCH_RVA),
        at(PIXELS_RVA),
        at(TILES_RVA),
        at(BLACK_RVA),
        at(GAP_RVA),
    ];
    let saved = capture(&kept);
    // Tiles of the test's: every third one there, a pattern each.
    let mut tile_data: Vec<u8> = (0..TILE * TILE * 4 * 16)
        .map(|n| (n * 37 + 5) as u8)
        .collect();
    let mut table = vec![0u32; (TILES * TILES) as usize];
    for (k, entry) in table.iter_mut().enumerate() {
        if k % 3 != 1 {
            *entry = tile_data.as_mut_ptr() as u32 + (k as u32 % 16) * TILE * TILE * 4 / 16;
        }
    }
    for bytes in [2u32, 4] {
        let pitch = 128 * bytes + 12;
        let mut surface = vec![0xAAu8; (pitch * 130) as usize];
        let s = surface.as_mut_ptr() as u32;
        for (x, y, fails) in [
            (0u32, 0u32, false),
            (5, 9, false),
            (0x7C, 0x7A, false),
            (0xFFFF_FFFE, 3, false),
            (0x90, 0x90, false),
            (2, 2, true),
        ] {
            let regions: Vec<Region> = vec![(s, surface.len()), at(GAP_RVA)];
            run_both_from(
                &mut t,
                &format!("{bytes} bytes a pixel, first tile ({x:#x}, {y:#x}), lock fails {fails}"),
                &copy,
                &regions,
                &|| unsafe {
                    std::ptr::write_bytes(s as *mut u8, 0xAA, (pitch * 130) as usize);
                    game_write(BYTES_RVA, bytes);
                    game_write(PITCH_RVA, pitch);
                    game_write(PIXELS_RVA, s);
                    game_write(TILES_RVA, table.as_ptr() as u32);
                    game_write(BLACK_RVA, 0x0012_3456u32);
                    game_write(GAP_RVA, 0u32);
                    LOCK_FAILS.store(fails, Ordering::SeqCst);
                },
                Regs {
                    eax: 0x33,
                    ecx: x,
                    edx: y,
                    ..Regs::SENTINEL
                },
                &|r| unsafe { blit_map_tiles(r, &mut test_part) },
            );
        }
        drop(surface);
    }
    LOCK_FAILS.store(false, Ordering::SeqCst);
    restore(&kept, &saved);
    drop((tile_data, table));
    t.finish()
}
