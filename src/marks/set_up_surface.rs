//! `marks::set_up_surface` (0x4CB0BC): `init_marks`' step 12: the surface picture
//! (`files\m\surfdata.pcx`, its alpha the inverted third byte of `files\m\surfmask.pcx`)
//! drawn into a 256 x 256 texture surface; a ring of 0x400 records of 0x94 bytes; two
//! tables of texture places (0x40 cells of an 8 x 8 grid, and 0x40 of 4 x 4 sub-cells).
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! The surface's lock and unlock are the game's (`sub_41AA4B` / `sub_41AB02`, DirectDraw).
//! Callers: `marks::init_marks` and code at 0x4D1BC8 (not in a function). Refuses memory
//! not had; a surface under 256 x 256 or a picture not read, after the lock (unlocked
//! again, the 0x40000 block left taken).

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::gfx::picture_store::PICTURE;
use crate::marks::blocks::take;
use crate::marks::load_mask_pictures::{lock, put_pixel};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;
use crate::scene::kind_texture_store::KIND_TEXTURES;

const MASK_LIST_RVA: Slot = Slot::at(0x93A1DA); // stru_93A1DA
const MASK_BYTES: u32 = 0x4_0000;
const SURFMASK_RVA: Slot = Slot::at(0x935751); // "files\m\surfmask.pcx"
const SURFDATA_RVA: Slot = Slot::at(0x93573C); // "files\m\surfdata.pcx"
const PIXELS: u32 = 0x1_0000;
const SURFACE_WIDTH_RVA: usize = rva(0x4F49A0); // g_surface.width
const SURFACE_HEIGHT_RVA: usize = rva(0x4F49A4); // g_surface.height
const SIDE: u32 = 0x100;
const RING_LIST_RVA: Slot = Slot::at(0x935620); // stru_935620
const RING_BYTES: u32 = 0x2_5000;
pub const RING_RVA: Slot = Slot::at(0x935668); // g_marksRingBlock (g_marksRing is 0x93A5C8)
pub const RING_CURSOR_RVA: Slot = Slot::at(0x935660); // g_marksRingCursor
pub const RING_COUNT_RVA: Slot = Slot::at(0x9356C8); // g_marksRingCount
const RING: u32 = 0x400;
const RECORD: u32 = 0x94;
const CELLS_RVA: Slot = Slot::at(0x935766); // g_marksCellPlaces: 0x40 of four floats
const SUBCELLS_RVA: Slot = Slot::at(0x935B66); // g_marksSubcellPlaces: 0x40 of 0x100 bytes
const CELLS: u32 = 0x40;
/// flt_7651C0 (1/8), flt_7651C8 (1/512), flt_7651CC (1/256), flt_7651D0 (1/32).
const EIGHTH_RVA: Slot = Slot::at(0x7651C0);
const EDGE_RVA: Slot = Slot::at(0x7651C8);
const PIXEL_RVA: Slot = Slot::at(0x7651CC);
const STEP_RVA: Slot = Slot::at(0x7651D0);

/// `gfx::read_picture_file` at 256 x 256 (the picture's sizes kept round it): whether it
/// was read.
unsafe fn read(path: Slot) -> Result<bool, Refused> {
    unsafe {
        let sizes: [u32; 3] = [
            PICTURE.width.get(),
            PICTURE.height.get(),
            PICTURE.row_bytes.get(),
        ];
        PICTURE.width.set(SIDE);
        PICTURE.height.set(SIDE);
        PICTURE.row_bytes.set(0xFFFF_FFFEu32);
        let read = crate::gfx::read_picture_file::read_picture_file(game_addr(path) as *mut u8);
        PICTURE.width.set(sizes[0]);
        PICTURE.height.set(sizes[1]);
        PICTURE.row_bytes.set(sizes[2]);
        read
    }
}

/// Unlocks the surface and refuses with `why`.
unsafe fn unlock_refusing(why: &'static str) -> Result<(), Refused> {
    unsafe { lock(KIND_TEXTURES.kind_first_textures[211].bits(), false)? };
    Err(BadInput(why).into())
}

/// The Rust version (see the header): `memory::allocate`, `d3d::lock_texture` /
/// `unlock_texture`, `gfx::read_picture_file`, `gfx::put_surface_pixel`,
/// `scene::forget_slot_texture`, `memory::free_list`, `nullsub_33` (nothing).
pub unsafe fn set_up_surface() -> Result<(), Refused> {
    unsafe {
        let block = take(MASK_LIST_RVA, MASK_BYTES)?;
        lock(KIND_TEXTURES.kind_first_textures[211].bits(), true)?;
        if game_read::<u32>(SURFACE_WIDTH_RVA) < SIDE || game_read::<u32>(SURFACE_HEIGHT_RVA) < SIDE
        {
            return unlock_refusing("a surface under 256 x 256");
        }
        if !read(SURFMASK_RVA)? {
            return unlock_refusing("surfmask.pcx was not read");
        }
        std::ptr::copy(
            PICTURE.buffer.get() as *const u8,
            block as *mut u8,
            4 * PIXELS as usize,
        );
        if !read(SURFDATA_RVA)? {
            return unlock_refusing("surfdata.pcx was not read");
        }
        let picture: u32 = PICTURE.buffer.get();
        for n in 0..PIXELS {
            let data = ((picture + 4 * n) as *const u32).read_unaligned();
            let mask = ((block + 4 * n) as *const u32).read_unaligned();
            let alpha = ((mask << 8) & 0xFF00_0000) ^ 0xFF00_0000;
            put_pixel((data & 0xFF_FFFF) | alpha, n & 0xFF, n >> 8)?;
        }
        lock(KIND_TEXTURES.kind_first_textures[211].bits(), false)?;
        crate::scene::forget_slot_texture::forget_slot_texture(
            KIND_TEXTURES.kind_first_slots[211].bits(),
        )?;
        crate::memory::free_list::free_list(
            game_addr(MASK_LIST_RVA) as *mut crate::memory::MemoryList
        )?;

        let ring = take(RING_LIST_RVA, RING_BYTES)?;
        game_write(RING_RVA, ring);
        game_write(RING_CURSOR_RVA, ring);
        game_write(RING_COUNT_RVA, 0u32);
        for i in 0..RING {
            let before = (i.wrapping_sub(1) & (RING - 1)) * RECORD + ring;
            let after = ((i + 1) & (RING - 1)) * RECORD + ring;
            ((i * RECORD + ring) as *mut u32).write_unaligned(before);
            ((i * RECORD + ring + 4) as *mut u32).write_unaligned(after);
        }

        let [eighth, edge, pixel, step]: [u32; 4] =
            [EIGHTH_RVA, EDGE_RVA, PIXEL_RVA, STEP_RVA].map(|r| game_read(r));
        let put = |at: u32, v: u32| (at as *mut u32).write_unaligned(v);
        let get = |at: u32| (at as *const u32).read_unaligned();
        let cells = game_addr(CELLS_RVA) as u32;
        for k in (0..CELLS).rev() {
            let at = cells + k * 0x10;
            let (u0, u1) = x87::int_times_plus_on((k & 7) as i32, eighth, edge, eighth, pixel);
            put(at, u0);
            put(at + 8, u1);
            let (v0, v1) = x87::int_times_plus_on((k >> 3) as i32, eighth, edge, eighth, pixel);
            put(at + 4, v0);
            put(at + 0xC, v1);
        }
        // Each cell's 4 rows of 4 sub-cells: 0x100 bytes.
        let sub = game_addr(SUBCELLS_RVA) as u32;
        for j in (0..CELLS).rev() {
            let mut at = sub + j * 0x100;
            let (u0, u1) = x87::int_times_plus_on((j & 7) as i32, eighth, edge, step, pixel);
            put(at, u0);
            put(at + 8, u1);
            let (v0, v1) = x87::int_times_plus_on((j >> 3) as i32, eighth, edge, step, pixel);
            put(at + 4, v0);
            put(at + 0xC, v1);
            let across = |at: &mut u32| {
                let p = *at;
                put(p + 0x10, x87::add(get(p), step));
                put(p + 0x14, x87::loaded(get(p + 4)));
                put(p + 0x18, x87::add(get(p + 8), step));
                put(p + 0x1C, x87::loaded(get(p + 0xC)));
                *at += 0x10;
            };
            for _ in 0..3 {
                across(&mut at);
            }
            for _ in 0..3 {
                let p = at;
                put(p + 0x10, x87::sub_thrice(get(p), step));
                put(p + 0x14, x87::add(get(p + 4), step));
                put(p + 0x18, x87::sub_thrice(get(p + 8), step));
                put(p + 0x1C, x87::add(get(p + 0xC), step));
                at += 0x10;
                for _ in 0..3 {
                    across(&mut at);
                }
            }
        }
    }
    Ok(())
}
