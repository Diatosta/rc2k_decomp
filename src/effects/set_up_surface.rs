//! `effects::set_up_surface` (0x4CB0BC): `init_marks`' step 12: the surface picture
//! (`files\m\surfdata.pcx`, its alpha the inverted third byte of `files\m\surfmask.pcx`)
//! drawn into a 256 x 256 texture surface; a ring of 0x400 records of 0x94 bytes; two
//! tables of texture places (0x40 cells of an 8 x 8 grid, and 0x40 of 4 x 4 sub-cells).
//!
//! The surface's lock and unlock are the game's (`d3d::lock_texture` / `d3d::unlock_texture`,
//! DirectDraw). Callers: `effects::init_marks` and code at 0x4D1BC8 (not in a function). Refuses
//! memory not had; a surface under 256 x 256 or a picture not read, after the lock (unlocked
//! again, the 0x40000 block left taken).

use crate::effects::load_mask_pictures::{lock, put_pixel};
use crate::effects::mask_store::Masks;
#[cfg(test)]
use crate::effects::owner::effects;
use crate::effects::ring_store::Ring;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::session::{Bytes, Dword};

const MASK_BYTES: u32 = 0x4_0000;
const PIXELS: u32 = 0x1_0000;
const SIDE: u32 = 0x100;
const CELLS: u32 = 0x40;
/// `gfx::read_picture_file` at 256 x 256 (the picture's sizes kept round it): whether it
/// was read.
fn read(path: &Bytes<21>) -> Result<bool, Refused> {
    let gfx = crate::gfx::owner::gfx();
    let sizes: [u32; 3] = [
        gfx.pictures.picture.width.get(),
        gfx.pictures.picture.height.get(),
        gfx.pictures.picture.row_bytes.get(),
    ];
    gfx.pictures.picture.width.set(SIDE);
    gfx.pictures.picture.height.set(SIDE);
    gfx.pictures.picture.row_bytes.set(0xFFFF_FFFEu32);
    let read = crate::gfx::read_picture_file::read_picture_file(path, gfx);
    gfx.pictures.picture.width.set(sizes[0]);
    gfx.pictures.picture.height.set(sizes[1]);
    gfx.pictures.picture.row_bytes.set(sizes[2]);
    read
}

/// Unlocks the surface and refuses with `why`.
fn unlock_refusing(why: &'static str, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    lock(scene.textures.kinds.kind_first_textures[211].bits(), false)?;
    Err(BadInput(why).into())
}

/// `memory::allocate`, `d3d::lock_texture` /
/// `unlock_texture`, `gfx::read_picture_file`, `gfx::put_surface_pixel`,
/// `scene::forget_slot_texture`, `memory::free_list`, `nullsub_33` (nothing).
pub fn set_up_surface(
    ring: &Ring,
    masks: &Masks,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    // The mask's copy is Rust's memory (phase 12), counted on the list as the game's
    // block was.
    let mut mask = crate::memory::owned::counted(&masks.list.list, MASK_BYTES)?;
    lock(scene.textures.kinds.kind_first_textures[211].bits(), true)?;
    if gfx.drawing.draw_surface.width.get() < SIDE || gfx.drawing.draw_surface.height.get() < SIDE {
        return unlock_refusing("a surface under 256 x 256", scene);
    }
    if !read(&ring.names.surfmask)? {
        return unlock_refusing("surfmask.pcx was not read", scene);
    }
    mask[..4 * PIXELS as usize].copy_from_slice(&crate::gfx::picture::read_bytes(
        gfx.pictures.picture.buffer.get(),
        0,
        4 * PIXELS as usize,
    )?);
    if !read(&ring.names.surfdata)? {
        return unlock_refusing("surfdata.pcx was not read", scene);
    }
    let picture =
        crate::gfx::picture::read_bytes(gfx.pictures.picture.buffer.get(), 0, 4 * PIXELS as usize)?;
    for n in 0..PIXELS {
        let at = 4 * n as usize;
        let data = u32::from_le_bytes(picture[at..at + 4].try_into().expect("four bytes"));
        let mask = u32::from_le_bytes(mask[at..at + 4].try_into().expect("four bytes"));
        let alpha = ((mask << 8) & 0xFF00_0000) ^ 0xFF00_0000;
        put_pixel((data & 0xFF_FFFF) | alpha, n & 0xFF, n >> 8)?;
    }
    lock(scene.textures.kinds.kind_first_textures[211].bits(), false)?;
    crate::scene::forget_slot_texture::forget_slot_texture(
        scene.textures.kinds.kind_first_slots[211].bits(),
        scene,
    )?;
    crate::memory::free_list::free_list(&masks.list.list)?;

    crate::effects::ring_entries::take(ring, &ring.work.list)?;
    ring.work.block.set(1);
    ring.work.cursor.set(1);
    ring.work.count.set(0);

    let [eighth, edge, pixel, step]: [u32; 4] = [
        setup.floats.eighth.bits(),
        setup.floats.surface_edge.bits(),
        setup.floats.surface_pixel.bits(),
        setup.floats.thirty_second.bits(),
    ];
    let places = &masks.surface.places;
    let end = size_of_val(places);
    let put = |at: u32, v: u32| {
        if at as usize + 4 > end {
            return Err(BadInput("a write past the surface places"));
        }
        places.set_u32(at as usize, v);
        Ok(())
    };
    let get = |at: u32| {
        if at as usize + 4 > end {
            return Err(BadInput("a read past the surface places"));
        }
        Ok(places.get_u32(at as usize))
    };
    let cells = 0u32;
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
    Ok(())
}

/// The surface masks' memory list and the bytes after it (IDA 0x93A1DA .. 0x93A21A, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MaskList {
    /// 0x93A1DA: `stru_93A1DA`: a `memory::List` (head, total, cursor, remaining)
    pub list: [Dword; 4],
    /// 0x93A1EA: Not named: 48 bytes (a gap, or unreferenced data)
    pub _gap_93a1ea: Bytes<48>,
}

/// MaskList's first values.
impl MaskList {
    pub const fn new() -> Self {
        MaskList {
            list: [
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
            ],
            _gap_93a1ea: Bytes::new([0; 48]),
        }
    }
}

impl MaskList {
    pub const START: usize = 0x93A1DA;
    pub const LEN: usize = 0x40;
}

#[cfg(test)]
mod mask_list_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn mask_list_starts_as_the_games() {
        assert_eq!(size_of::<MaskList>(), MaskList::LEN);
        assert!(
            bytes(&effects().masks.list)
                == crate::moved::image_bytes(MaskList::START, MaskList::LEN),
            "MASK_LIST starts differently from the image"
        );
        assert_eq!(offset_of!(MaskList, list), 0x93A1DA - MaskList::START);
    }
}
