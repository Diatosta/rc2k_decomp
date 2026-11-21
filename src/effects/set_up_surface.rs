//! `effects::set_up_surface`: `init_marks`' step 12: the surface picture (`files\m\surfdata.pcx`,
//! its alpha the inverted third byte of `files\m\surfmask.pcx`) drawn into a 256 x 256 texture
//! surface; a ring of 0x400 records of 0x94 bytes; two tables of texture places (0x40 cells of an 8
//! x 8 grid, and 0x40 of 4 x 4 sub-cells).
//!
//! The surface's lock and unlock are `d3d::lock_texture` / `d3d::unlock_texture`. Refuses memory
//! not had; a surface under 256 x 256 or a picture not read, after the lock (unlocked again, the
//! 0x40000 block left taken).

use crate::effects::load_mask_pictures::{lock, put_pixel};
use crate::effects::mask_store::Masks;
use crate::effects::ring_store::Ring;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::memory::session::PlainBytes;

const MASK_BYTES: u32 = 0x4_0000;
const PIXELS: u32 = 0x1_0000;
const SIDE: u32 = 0x100;
const CELLS: u32 = 0x40;
/// `gfx::read_picture_file` at 256 x 256 (the picture's sizes kept round it): whether it
/// was read.
fn read(
    path: &PlainBytes<21>,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<bool, Refused> {
    let sizes: [u32; 3] = [
        gfx.pictures.picture.width.get(),
        gfx.pictures.picture.height.get(),
        gfx.pictures.picture.row_bytes.get(),
    ];
    gfx.pictures.picture.width.set(SIDE);
    gfx.pictures.picture.height.set(SIDE);
    gfx.pictures.picture.row_bytes.set(0xFFFF_FFFEu32);
    let read = crate::gfx::read_picture_file::read_picture_file(path, gfx, system);
    gfx.pictures.picture.width.set(sizes[0]);
    gfx.pictures.picture.height.set(sizes[1]);
    gfx.pictures.picture.row_bytes.set(sizes[2]);
    read
}

/// Unlocks the surface and refuses with `why`.
fn unlock_refusing(
    why: &'static str,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    lock(gfx, scene.textures.first_textures()[211], false)?;
    Err(BadInput(why).into())
}

/// `memory::allocate`, `d3d::lock_texture` / `unlock_texture`, `gfx::read_picture_file`,
/// `gfx::put_surface_pixel`, `scene::forget_slot_texture`, `memory::free_list`.
pub(super) fn set_up_surface(
    ring: &Ring,
    masks: &Masks,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    // The mask's copy is Rust's memory, counted on the list.
    let mut mask = crate::memory::owned::counted(&masks.list.list, MASK_BYTES)?;
    lock(gfx, scene.textures.first_textures()[211], true)?;
    if gfx.drawing.draw_surface.width.get() < SIDE || gfx.drawing.draw_surface.height.get() < SIDE {
        return unlock_refusing("a surface under 256 x 256", scene, gfx);
    }
    if !read(&ring.names.surfmask, gfx, system)? {
        return unlock_refusing("surfmask.pcx was not read", scene, gfx);
    }
    mask[..4 * PIXELS as usize].copy_from_slice(&crate::gfx::picture::read_bytes(
        &gfx.pictures.buffers,
        gfx.pictures.picture.buffer.get(),
        0,
        4 * PIXELS as usize,
    )?);
    if !read(&ring.names.surfdata, gfx, system)? {
        return unlock_refusing("surfdata.pcx was not read", scene, gfx);
    }
    let picture = crate::gfx::picture::read_bytes(
        &gfx.pictures.buffers,
        gfx.pictures.picture.buffer.get(),
        0,
        4 * PIXELS as usize,
    )?;
    for n in 0..PIXELS {
        let at = 4 * n as usize;
        let data = u32::from_le_bytes(picture[at..at + 4].try_into().expect("four bytes"));
        let mask = u32::from_le_bytes(mask[at..at + 4].try_into().expect("four bytes"));
        let alpha = ((mask << 8) & 0xFF00_0000) ^ 0xFF00_0000;
        put_pixel((data & 0xFF_FFFF) | alpha, n & 0xFF, n >> 8, gfx, system)?;
    }
    lock(gfx, scene.textures.first_textures()[211], false)?;
    crate::scene::forget_slot_texture::forget_slot_texture(
        scene.textures.kinds.borrow().kind_first_slots[211],
        scene,
    )?;
    crate::memory::free_list::free_list(&masks.list.list)?;

    crate::effects::ring_entries::take(ring, &ring.work)?;
    ring.work.borrow_mut().block = 1;
    ring.work.borrow_mut().cursor = 1;
    ring.work.borrow_mut().count = 0;

    let [eighth, edge, pixel, step]: [f32; 4] = [
        setup.floats.eighth,
        setup.floats.surface_edge,
        setup.floats.surface_pixel,
        setup.floats.thirty_second,
    ];
    let end = masks.surface.borrow().places.len();
    let put = |at: u32, v: u32| {
        if at as usize + 4 > end {
            return Err(BadInput("a write past the surface places"));
        }
        masks.surface.borrow_mut().places.set_u32(at as usize, v);
        Ok(())
    };
    let get = |at: u32| {
        if at as usize + 4 > end {
            return Err(BadInput("a read past the surface places"));
        }
        Ok(masks.surface.borrow().places.get_u32(at as usize))
    };
    let cells = 0u32;
    for k in (0..CELLS).rev() {
        let at = cells + k * 0x10;
        let (u0, u1) = x87::int_times_plus_on((k & 7) as i32, eighth, edge, eighth, pixel);
        put(at, u0.to_bits())?;
        put(at + 8, u1.to_bits())?;
        let (v0, v1) = x87::int_times_plus_on((k >> 3) as i32, eighth, edge, eighth, pixel);
        put(at + 4, v0.to_bits())?;
        put(at + 0xC, v1.to_bits())?;
    }
    // Each cell's 4 rows of 4 sub-cells: 0x100 bytes.
    let sub = cells + 0x400;
    for j in (0..CELLS).rev() {
        let mut at = sub + j * 0x100;
        let (u0, u1) = x87::int_times_plus_on((j & 7) as i32, eighth, edge, step, pixel);
        put(at, u0.to_bits())?;
        put(at + 8, u1.to_bits())?;
        let (v0, v1) = x87::int_times_plus_on((j >> 3) as i32, eighth, edge, step, pixel);
        put(at + 4, v0.to_bits())?;
        put(at + 0xC, v1.to_bits())?;
        let across = |at: &mut u32| -> Result<_, Refused> {
            let p = *at;
            put(p + 0x10, x87::add(get(p)?, step).to_bits())?;
            put(p + 0x14, x87::loaded(get(p + 4)?).to_bits())?;
            put(p + 0x18, x87::add(get(p + 8)?, step).to_bits())?;
            put(p + 0x1C, x87::loaded(get(p + 0xC)?).to_bits())?;
            *at += 0x10;
            Ok(())
        };
        for _ in 0..3 {
            across(&mut at)?;
        }
        for _ in 0..3 {
            let p = at;
            put(p + 0x10, x87::sub_thrice(get(p)?, step).to_bits())?;
            put(p + 0x14, x87::add(get(p + 4)?, step).to_bits())?;
            put(p + 0x18, x87::sub_thrice(get(p + 8)?, step).to_bits())?;
            put(p + 0x1C, x87::add(get(p + 0xC)?, step).to_bits())?;
            at += 0x10;
            for _ in 0..3 {
                across(&mut at)?;
            }
        }
    }
    Ok(())
}
