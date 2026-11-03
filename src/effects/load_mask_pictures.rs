//! `effects::load_mask_pictures` (0x4C8BD8): `init_marks`' step 10: the kind's two mask
//! pictures made 256 x 128 blocks of white with the mask as alpha, and drawn into a
//! texture surface.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! The ground picture's name is the `dword_934CC8`-th of six (`files\m\snow1.pcx`, `mud1`,
//! `dust1`, `mud1`, `mud1`, `gumbo0`); it is read twice and never used (section 16). The
//! surface's lock and unlock are the game's (`sub_41AA4B` / `sub_41AB02`, DirectDraw). One
//! caller, `effects::init_marks`. Refuses a ground past the six names (before anything), a
//! surface wider than 256 (after the lock, unlocked again; the three blocks are left
//! taken), memory or a picture not had, as its steps do.

use crate::effects::kind_values::MARKS_KIND;
use crate::effects::mask_store::{
    GROUND_NAMES, KIND_PICTURES, MASK_BLOCKS, MASK_PIXELS, Mask, PICTURE_COPY,
};
use crate::errors::{BadInput, Refused};
use crate::gfx::picture_store::PICTURE;
use crate::scene::kind_texture_store::KIND_TEXTURES;

const NAMES: i32 = 6;
/// The mask blocks' bytes (each `MASK_PIXELS` dwords) and the picture copy's.
const MASK_BYTES: u32 = 0x2_0000;
const COPY_BYTES: u32 = 0x8_0000;
const MOST_KIND: i32 = 0x21;
/// The mask paths in `PICTURE_COPY.paths`: "files\m\mask00.pcx" and "files\m\bmask00.pcx"
/// (the third, "files\m\bdata00.pcx", is not read here), and where their two digits are.
const MASK_PATH_AT: usize = 0;
const BMASK_PATH_AT: usize = 0x13;
const DIGITS_AT: [usize; 3] = [0xC, 0x20, 0x34];
const MOST_WIDTH: u32 = 0x100;
const COPIED: u32 = 0x8000; // dwords
const ROWS: u32 = 0x7E;

/// `gfx::read_picture_file` at 256 x 128 (the picture's sizes kept round it); refused
/// when not read (the original then counts 0x1FFFFFFF down and leaves with its stack one
/// off).
fn read(path: crate::pointers::GamePtr) -> Result<(), Refused> {
    {
        use rc2k_core::strings::bytes::Bytes as _;
        let text = path.bytes("a mask picture's path")?;
        let mut k = 0;
        loop {
            match text.byte(k) {
                Some(0) => break,
                Some(_) => k += 1,
                None => return Err(BadInput("a mask picture's path runs past its region").into()),
            }
        }
    }
    // SAFETY: the path's NUL lies in its region (checked above).
    let sizes: [u32; 3] = [
        PICTURE.width.get(),
        PICTURE.height.get(),
        PICTURE.row_bytes.get(),
    ];
    PICTURE.width.set(0x100u32);
    PICTURE.height.set(0x80u32);
    PICTURE.row_bytes.set(0xFFFF_FFFEu32);
    let read = crate::gfx::read_picture_file::read_picture_file(&path);
    PICTURE.width.set(sizes[0]);
    PICTURE.height.set(sizes[1]);
    PICTURE.row_bytes.set(sizes[2]);
    if !read? {
        return Err(BadInput("a mask picture was not read").into());
    }
    Ok(())
}

/// `gfx::put_surface_pixel` (components, x, y).
pub fn put_pixel(components: u32, x: u32, y: u32) -> Result<(), Refused> {
    crate::gfx::put_surface_pixel::put_surface_pixel(components, x, y)?;
    Ok(())
}

/// `d3d::lock_texture` (`lock`) or `d3d::unlock_texture` of the texture slot `slot` (their
/// flags unread).
pub fn lock(slot: u32, lock: bool) -> Result<(), Refused> {
    if lock {
        crate::d3d::lock_texture::lock_texture(slot)?;
    } else {
        crate::d3d::unlock_texture::unlock_texture(slot)?;
    }
    Ok(())
}

/// A mask read, copied to the 0x80000 block (`copy`), the ground read, and the mask `to`
/// made and drawn at rows 0x80 .. 0xFD.
fn mask(path: usize, to: Mask, copy: &mut [u8], width: u32) -> Result<(), Refused> {
    {
        read(crate::pointers::GamePtr::to(&PICTURE_COPY.paths).offset(path as u32))?;
        copy[..4 * COPIED as usize].copy_from_slice(&crate::gfx::picture::read_bytes(
            PICTURE.buffer.get(),
            0,
            4 * COPIED as usize,
        )?);
        read(
            crate::pointers::GamePtr::to(&GROUND_NAMES.names)
                .offset(GROUND_NAMES.ground_path.get()),
        )?;
        for y in 0..ROWS {
            for x in 0..width.max(1) {
                let n = ((y << 8).wrapping_add(x) * 4) as usize;
                let mask = u32::from_le_bytes(copy[n..n + 4].try_into().expect("four bytes"));
                let pixel = (mask << 8) & 0xFF00_0000 | 0xFF_FFFF;
                to.pixels().with(|p| p[n / 4] = pixel)?;
                put_pixel(pixel, x, y + 0x80)?;
            }
        }
    }
    Ok(())
}

/// The Rust version: the ground's name skipped to (`g_marksGroundPath`); the three blocks
/// taken (`memory::allocate`); the surface locked (`d3d::lock_texture`); the kind (held
/// to 0 .. 0x21) its picture number's two digits in the three paths; each mask
/// (`gfx::read_picture_file`) into its block and drawn (`gfx::put_surface_pixel`);
/// unlocked, the texture slot forgotten (`scene::forget_slot_texture`), the 0x80000
/// block's list freed, `dword_934D6E` -1.
pub fn load_mask_pictures() -> Result<(), Refused> {
    let ground: i32 = GROUND_NAMES.ground.get() as i32;
    if ground >= NAMES {
        return Err(BadInput("a ground past the six names").into());
    }
    let mut name = 0usize;
    for _ in 0..ground.max(0) {
        while *GROUND_NAMES
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
    GROUND_NAMES.ground_path.set(name as u32);
    // The masks and the picture copy are Rust's (phase 12), counted on their lists
    // as the game's blocks were.
    for (mask, list) in [
        (
            Mask::Plain,
            crate::pointers::GamePtr::to(&MASK_BLOCKS.list_a),
        ),
        (
            Mask::Bordered,
            crate::pointers::GamePtr::to(&MASK_BLOCKS.list_b),
        ),
    ] {
        mask.pixels().take(list, MASK_BYTES, vec![0; MASK_PIXELS])?;
    }
    let mut copy = crate::memory::owned::counted(
        crate::pointers::GamePtr::to(&PICTURE_COPY.list),
        COPY_BYTES,
    )?;
    lock(KIND_TEXTURES.kind_first_textures[202].bits(), true)?;
    let width: u32 = crate::gfx::screen_state::DRAW_SURFACE.width.get();
    if width > MOST_WIDTH {
        lock(KIND_TEXTURES.kind_first_textures[202].bits(), false)?;
        return Err(BadInput("a mask surface wider than 256").into());
    }
    let kind = (MARKS_KIND.kind.bits() as i32).clamp(0, MOST_KIND) as u32;
    let number: u32 = KIND_PICTURES.numbers[kind as usize].get();
    let digits = [
        ((number / 10) as u8).wrapping_add(b'0'),
        (number % 10) as u8 + b'0',
    ];
    for d in DIGITS_AT {
        PICTURE_COPY.paths.set(d, digits[0]);
        PICTURE_COPY.paths.set(d + 1, digits[1]);
    }
    mask(MASK_PATH_AT, Mask::Plain, &mut copy, width)?;
    mask(BMASK_PATH_AT, Mask::Bordered, &mut copy, width)?;
    lock(KIND_TEXTURES.kind_first_textures[202].bits(), false)?;
    crate::scene::forget_slot_texture::forget_slot_texture(
        KIND_TEXTURES.kind_first_slots[202].bits(),
    )?;
    crate::memory::free_list::free_list(crate::pointers::GamePtr::to(&PICTURE_COPY.list))?;
    MASK_BLOCKS.seen.set(u32::MAX);
    Ok(())
}
