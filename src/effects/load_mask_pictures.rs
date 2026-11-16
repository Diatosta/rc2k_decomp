//! `effects::load_mask_pictures`: `init_marks`' step 10: the kind's two mask
//! pictures made 256 x 128 blocks of white with the mask as alpha, and drawn into a
//! texture surface.
//!
//! The ground picture's name is the `GroundNames.ground`-th of six (`files\m\snow1.pcx`, `mud1`,
//! `dust1`, `mud1`, `mud1`, `gumbo0`); it is read twice and never used (a slip kept as is). The
//! surface's lock and unlock are `d3d::lock_texture` / `d3d::unlock_texture`. Refuses a ground past
//! the six names (before anything), a surface wider than 256 (after the lock, unlocked again; the
//! three blocks are left taken), memory or a picture not had, as its steps do.

use crate::effects::mask_store::{KIND_PICTURES, MASK_PIXELS, Mask, Masks};
use crate::errors::{BadInput, Refused};

const NAMES: i32 = 6;
/// The mask blocks' bytes (each `MASK_PIXELS` dwords) and the picture copy's.
const MASK_BYTES: u32 = 0x2_0000;
const COPY_BYTES: u32 = 0x8_0000;
const MOST_KIND: i32 = 0x21;
/// The mask paths in `PictureCopy::paths`: "files\m\mask00.pcx" and "files\m\bmask00.pcx"
/// (the third, "files\m\bdata00.pcx", is not read here), and where their two digits are.
const MASK_PATH_AT: usize = 0;
const BMASK_PATH_AT: usize = 0x13;
const DIGITS_AT: [usize; 3] = [0xC, 0x20, 0x34];
const MOST_WIDTH: u32 = 0x100;
const COPIED: u32 = 0x8000; // dwords
const ROWS: u32 = 0x7E;

/// `gfx::read_picture_file` at 256 x 128 (the picture's sizes kept round it); refused when
/// not read.
fn read(
    path: &dyn crate::memory::session::TextPlace,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    path.text("a mask picture's path")?;
    let sizes: [u32; 3] = [
        gfx.pictures.picture.width.get(),
        gfx.pictures.picture.height.get(),
        gfx.pictures.picture.row_bytes.get(),
    ];
    gfx.pictures.picture.width.set(0x100u32);
    gfx.pictures.picture.height.set(0x80u32);
    gfx.pictures.picture.row_bytes.set(0xFFFF_FFFEu32);
    let read = crate::gfx::read_picture_file::read_picture_file(path, gfx, system);
    gfx.pictures.picture.width.set(sizes[0]);
    gfx.pictures.picture.height.set(sizes[1]);
    gfx.pictures.picture.row_bytes.set(sizes[2]);
    if !read? {
        return Err(BadInput("a mask picture was not read").into());
    }
    Ok(())
}

/// `gfx::put_surface_pixel` (components, x, y).
pub fn put_pixel(
    components: u32,
    x: u32,
    y: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    crate::gfx::put_surface_pixel::put_surface_pixel(components, x, y, gfx, system)?;
    Ok(())
}

/// `d3d::lock_texture` (`lock`) or `d3d::unlock_texture` of the texture slot `slot` (their
/// flags unread).
pub fn lock(gfx: &crate::gfx::owner::Gfx, slot: u32, lock: bool) -> Result<(), Refused> {
    if lock {
        crate::d3d::lock_texture::lock_texture_in(gfx, slot)?;
    } else {
        crate::d3d::unlock_texture::unlock_texture(gfx, slot)?;
    }
    Ok(())
}

/// A mask read, copied to the 0x80000 block (`copy`), the ground read, and the mask `to`
/// made and drawn at rows 0x80 .. 0xFD.
fn mask(
    masks: &Masks,
    path: usize,
    to: Mask,
    copy: &mut [u8],
    width: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    {
        // A file path (text up to its NUL) at a byte offset in the paths.
        read(&masks.picture_copy.paths.text_from(path), gfx, system)?;
        copy[..4 * COPIED as usize].copy_from_slice(&crate::gfx::picture::read_bytes(
            &gfx.pictures.buffers,
            gfx.pictures.picture.buffer.get(),
            0,
            4 * COPIED as usize,
        )?);
        // The ground's file path, a text up to its NUL, at the offset the settings name.
        read(
            &masks
                .ground_names
                .names
                .text_from(masks.ground_names.ground_path.get() as usize),
            gfx,
            system,
        )?;
        for y in 0..ROWS {
            for x in 0..width.max(1) {
                let n = ((y << 8).wrapping_add(x) * 4) as usize;
                let mask = u32::from_le_bytes(copy[n..n + 4].try_into().expect("four bytes"));
                let pixel = (mask << 8) & 0xFF00_0000 | 0xFF_FFFF;
                to.pixels(masks).with(|p| p[n / 4] = pixel)?;
                put_pixel(pixel, x, y + 0x80, gfx, system)?;
            }
        }
    }
    Ok(())
}

/// The ground's name skipped to; the three blocks taken; the surface locked (`d3d::lock_texture`);
/// the kind (held to 0 .. 0x21) its picture number's two digits in the three paths; each mask
/// (`gfx::read_picture_file`) into its block and drawn (`gfx::put_surface_pixel`); unlocked, the
/// texture slot forgotten (`scene::forget_slot_texture`), the 0x80000 block's list freed,
/// `MaskBlocks.seen` -1.
pub fn load_mask_pictures(
    masks: &Masks,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let ground: i32 = masks.ground_names.ground.get() as i32;
    if ground >= NAMES {
        return Err(BadInput("a ground past the six names").into());
    }
    let mut name = 0usize;
    for _ in 0..ground.max(0) {
        while *masks
            .ground_names
            .names
            .read_at(name, 1, "the ground names")?
            .first()
            .unwrap_or(&0)
            != 0
        {
            name += 1;
        }
        name += 1;
    }
    masks.ground_names.ground_path.set(name as u32);
    // The masks and the picture copy are Rust's, counted on their lists.
    for (mask, list) in [
        (Mask::Plain, &masks.blocks.list_a),
        (Mask::Bordered, &masks.blocks.list_b),
    ] {
        mask.pixels(masks)
            .take(list, MASK_BYTES, vec![0; MASK_PIXELS])?;
    }
    let mut copy = crate::memory::owned::counted(&masks.picture_copy.list, COPY_BYTES)?;
    lock(
        gfx,
        scene.textures.kinds.get().kind_first_textures[202].get(),
        true,
    )?;
    let width: u32 = gfx.drawing.draw_surface.width.get();
    if width > MOST_WIDTH {
        lock(
            gfx,
            scene.textures.kinds.get().kind_first_textures[202].get(),
            false,
        )?;
        return Err(BadInput("a mask surface wider than 256").into());
    }
    let kind = (effects.kinds.marks_kind.kind.get() as i32).clamp(0, MOST_KIND) as u32;
    let number: u32 = KIND_PICTURES.numbers[kind as usize];
    let digits = [
        ((number / 10) as u8).wrapping_add(b'0'),
        (number % 10) as u8 + b'0',
    ];
    for d in DIGITS_AT {
        masks.picture_copy.paths.set(d, digits[0]);
        masks.picture_copy.paths.set(d + 1, digits[1]);
    }
    mask(
        masks,
        MASK_PATH_AT,
        Mask::Plain,
        &mut copy,
        width,
        gfx,
        system,
    )?;
    mask(
        masks,
        BMASK_PATH_AT,
        Mask::Bordered,
        &mut copy,
        width,
        gfx,
        system,
    )?;
    lock(
        gfx,
        scene.textures.kinds.get().kind_first_textures[202].get(),
        false,
    )?;
    crate::scene::forget_slot_texture::forget_slot_texture(
        scene.textures.kinds.get().kind_first_slots[202].get(),
        scene,
    )?;
    crate::memory::free_list::free_list(&masks.picture_copy.list)?;
    masks.blocks.seen.set(u32::MAX);
    Ok(())
}
