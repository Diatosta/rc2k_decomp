//! `effects::set_up_surface` (0x4CB0BC): `init_marks`' step 12: the surface picture
//! (`files\m\surfdata.pcx`, its alpha the inverted third byte of `files\m\surfmask.pcx`)
//! drawn into a 256 x 256 texture surface; a ring of 0x400 records of 0x94 bytes; two
//! tables of texture places (0x40 cells of an 8 x 8 grid, and 0x40 of 4 x 4 sub-cells).
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! The surface's lock and unlock are the game's (`sub_41AA4B` / `sub_41AB02`, DirectDraw).
//! Callers: `effects::init_marks` and code at 0x4D1BC8 (not in a function). Refuses memory
//! not had; a surface under 256 x 256 or a picture not read, after the lock (unlocked
//! again, the 0x40000 block left taken).

use crate::effects::load_mask_pictures::{lock, put_pixel};
use crate::effects::ring_store::{PICTURE_NAMES, RING_WORK};
use crate::effects::surface_store::SURFACE;
use crate::effects::tuning::MARKS_FLOATS;
use crate::errors::{BadInput, Refused};
use crate::gfx::picture_store::PICTURE;
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::session::{Bytes, Dword};

const MASK_BYTES: u32 = 0x4_0000;
const PIXELS: u32 = 0x1_0000;
const SIDE: u32 = 0x100;
const CELLS: u32 = 0x40;
/// `gfx::read_picture_file` at 256 x 256 (the picture's sizes kept round it): whether it
/// was read.
fn read(path: &Bytes<21>) -> Result<bool, Refused> {
    unsafe {
        let sizes: [u32; 3] = [
            PICTURE.width.get(),
            PICTURE.height.get(),
            PICTURE.row_bytes.get(),
        ];
        PICTURE.width.set(SIDE);
        PICTURE.height.set(SIDE);
        PICTURE.row_bytes.set(0xFFFF_FFFEu32);
        let read = crate::gfx::read_picture_file::read_picture_file(path.as_ptr());
        PICTURE.width.set(sizes[0]);
        PICTURE.height.set(sizes[1]);
        PICTURE.row_bytes.set(sizes[2]);
        read
    }
}

/// Unlocks the surface and refuses with `why`.
fn unlock_refusing(why: &'static str) -> Result<(), Refused> {
    lock(KIND_TEXTURES.kind_first_textures[211].bits(), false)?;
    Err(BadInput(why).into())
}

/// The Rust version (see the header): `memory::allocate`, `d3d::lock_texture` /
/// `unlock_texture`, `gfx::read_picture_file`, `gfx::put_surface_pixel`,
/// `scene::forget_slot_texture`, `memory::free_list`, `nullsub_33` (nothing).
pub fn set_up_surface() -> Result<(), Refused> {
    unsafe {
        // The mask's copy is Rust's memory (phase 12), counted on the list as the game's
        // block was.
        let mut mask = crate::memory::owned::counted(
            MASK_LIST.list[0].as_ptr() as *mut crate::memory::MemoryList,
            MASK_BYTES,
        )?;
        lock(KIND_TEXTURES.kind_first_textures[211].bits(), true)?;
        if crate::gfx::screen_state::DRAW_SURFACE.width.get() < SIDE
            || crate::gfx::screen_state::DRAW_SURFACE.height.get() < SIDE
        {
            return unlock_refusing("a surface under 256 x 256");
        }
        if !read(&PICTURE_NAMES.surfmask)? {
            return unlock_refusing("surfmask.pcx was not read");
        }
        mask[..4 * PIXELS as usize].copy_from_slice(&crate::gfx::picture::read_bytes(
            PICTURE.buffer.get(),
            0,
            4 * PIXELS as usize,
        )?);
        if !read(&PICTURE_NAMES.surfdata)? {
            return unlock_refusing("surfdata.pcx was not read");
        }
        let picture =
            crate::gfx::picture::read_bytes(PICTURE.buffer.get(), 0, 4 * PIXELS as usize)?;
        for n in 0..PIXELS {
            let at = 4 * n as usize;
            let data = u32::from_le_bytes(picture[at..at + 4].try_into().expect("four bytes"));
            let mask = u32::from_le_bytes(mask[at..at + 4].try_into().expect("four bytes"));
            let alpha = ((mask << 8) & 0xFF00_0000) ^ 0xFF00_0000;
            put_pixel((data & 0xFF_FFFF) | alpha, n & 0xFF, n >> 8)?;
        }
        lock(KIND_TEXTURES.kind_first_textures[211].bits(), false)?;
        crate::scene::forget_slot_texture::forget_slot_texture(
            KIND_TEXTURES.kind_first_slots[211].bits(),
        )?;
        crate::memory::free_list::free_list(
            MASK_LIST.list[0].as_ptr() as *mut crate::memory::MemoryList
        )?;

        crate::effects::ring_entries::take(
            RING_WORK.list.as_ptr() as *mut crate::memory::MemoryList
        )?;
        RING_WORK.block.set(1);
        RING_WORK.cursor.set(1);
        RING_WORK.count.set(0);

        let [eighth, edge, pixel, step]: [u32; 4] = [
            MARKS_FLOATS.eighth.bits(),
            MARKS_FLOATS.over_512.bits(),
            MARKS_FLOATS.over_256.bits(),
            MARKS_FLOATS.over_32.bits(),
        ];
        let put = |at: u32, v: u32| GamePtr::new(at).write::<u32>(v, "a mark's field");
        let get = |at: u32| GamePtr::new(at).read::<u32>("a mark's field");
        let cells = SURFACE.places.addr();
        for k in (0..CELLS).rev() {
            let at = cells + k * 0x10;
            let (u0, u1) = x87::int_times_plus_on((k & 7) as i32, eighth, edge, eighth, pixel);
            put(at, u0)?;
            put(at + 8, u1)?;
            let (v0, v1) = x87::int_times_plus_on((k >> 3) as i32, eighth, edge, eighth, pixel);
            put(at + 4, v0)?;
            put(at + 0xC, v1)?;
        }
        // Each cell's 4 rows of 4 sub-cells: 0x100 bytes.
        let sub = cells + 0x400;
        for j in (0..CELLS).rev() {
            let mut at = sub + j * 0x100;
            let (u0, u1) = x87::int_times_plus_on((j & 7) as i32, eighth, edge, step, pixel);
            put(at, u0)?;
            put(at + 8, u1)?;
            let (v0, v1) = x87::int_times_plus_on((j >> 3) as i32, eighth, edge, step, pixel);
            put(at + 4, v0)?;
            put(at + 0xC, v1)?;
            let across = |at: &mut u32| -> Result<_, Refused> {
                let p = *at;
                put(p + 0x10, x87::add(get(p)?, step))?;
                put(p + 0x14, x87::loaded(get(p + 4)?))?;
                put(p + 0x18, x87::add(get(p + 8)?, step))?;
                put(p + 0x1C, x87::loaded(get(p + 0xC)?))?;
                *at += 0x10;
                Ok(())
            };
            for _ in 0..3 {
                across(&mut at)?;
            }
            for _ in 0..3 {
                let p = at;
                put(p + 0x10, x87::sub_thrice(get(p)?, step))?;
                put(p + 0x14, x87::add(get(p + 4)?, step))?;
                put(p + 0x18, x87::sub_thrice(get(p + 8)?, step))?;
                put(p + 0x1C, x87::add(get(p + 0xC)?, step))?;
                at += 0x10;
                for _ in 0..3 {
                    across(&mut at)?;
                }
            }
        }
    }
    Ok(())
}

/// The surface masks' memory list and the bytes after it (IDA 0x93A1DA .. 0x93A21A, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct MaskList {
    /// 0x93A1DA: `stru_93A1DA`: a `memory::List` (head, total, cursor, remaining)
    pub list: [Dword; 4],
    /// 0x93A1EA: Not named: 48 bytes (a gap, or unreferenced data)
    pub _gap_93a1ea: Bytes<48>,
}

/// MaskList's first values.
pub static MASK_LIST: MaskList = MaskList {
    list: [
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
    ],
    _gap_93a1ea: Bytes::new([0; 48]),
};

impl MaskList {
    pub const START: usize = 0x93A1DA;
    pub const LEN: usize = 0x40;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&MASK_LIST as *const MaskList) as usize
    }
}

#[cfg(test)]
mod mask_list_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = v as *const T as *const u8;
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn mask_list_starts_as_the_games() {
        assert_eq!(size_of::<MaskList>(), MaskList::LEN);
        assert!(
            bytes(&MASK_LIST) == crate::moved::image_bytes(MaskList::START, MaskList::LEN),
            "MASK_LIST starts differently from the image"
        );
        assert_eq!(offset_of!(MaskList, list), 0x93A1DA - MaskList::START);
    }
}
