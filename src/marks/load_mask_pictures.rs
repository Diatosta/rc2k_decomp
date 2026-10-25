//! `marks::load_mask_pictures` (0x4C8BD8): `init_marks`' step 10: the kind's two mask
//! pictures made 256 x 128 blocks of white with the mask as alpha, and drawn into a
//! texture surface.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! The ground picture's name is the `dword_934CC8`-th of six (`files\m\snow1.pcx`, `mud1`,
//! `dust1`, `mud1`, `mud1`, `gumbo0`); it is read twice and never used (section 16). The
//! surface's lock and unlock are the game's (`sub_41AA4B` / `sub_41AB02`, DirectDraw). One
//! caller, `marks::init_marks`. Refuses a ground past the six names (before anything), a
//! surface wider than 256 (after the lock, unlocked again; the three blocks are left
//! taken), memory or a picture not had, as its steps do.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::gfx::picture_store::PICTURE;
use crate::marks::kind_values::MARKS_KIND;
use crate::marks::mask_store::{GROUND_NAMES, KIND_PICTURES, MASK_BLOCKS, PICTURE_COPY};
use crate::marks::store::game_read;
use crate::memory::MemoryList;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::session::Dword;

const NAMES: i32 = 6;
/// The blocks: bytes, their list, where the pointer goes.
fn blocks() -> [(u32, *mut MemoryList, &'static Dword); 3] {
    [
        (0x2_0000, MASK_BLOCKS.list_a.as_ptr(), &MASK_BLOCKS.mask), // g_marksMaskBlock
        (0x2_0000, MASK_BLOCKS.list_b.as_ptr(), &MASK_BLOCKS.bmask), // g_marksBMaskBlock
        (0x8_0000, PICTURE_COPY.list.as_ptr(), &PICTURE_COPY.copy), // g_marksPictureCopy
    ]
}
const MOST_KIND: i32 = 0x21;
/// The mask paths in `PICTURE_COPY.paths`: "files\m\mask00.pcx" and "files\m\bmask00.pcx"
/// (the third, "files\m\bdata00.pcx", is not read here), and where their two digits are.
const MASK_PATH_AT: usize = 0;
const BMASK_PATH_AT: usize = 0x13;
const DIGITS_AT: [usize; 3] = [0xC, 0x20, 0x34];
const SURFACE_WIDTH_RVA: usize = rva(0x4F49A0); // g_surface.width
const MOST_WIDTH: u32 = 0x100;
const COPIED: u32 = 0x8000; // dwords
const ROWS: u32 = 0x7E;

/// `gfx::read_picture_file` at 256 x 128 (the picture's sizes kept round it); refused
/// when not read (the original then counts 0x1FFFFFFF down and leaves with its stack one
/// off).
unsafe fn read(path: u32) -> Result<(), Refused> {
    unsafe {
        let sizes: [u32; 3] = [
            PICTURE.width.get(),
            PICTURE.height.get(),
            PICTURE.row_bytes.get(),
        ];
        PICTURE.width.set(0x100u32);
        PICTURE.height.set(0x80u32);
        PICTURE.row_bytes.set(0xFFFF_FFFEu32);
        let read = crate::gfx::read_picture_file::read_picture_file(path as *mut u8);
        PICTURE.width.set(sizes[0]);
        PICTURE.height.set(sizes[1]);
        PICTURE.row_bytes.set(sizes[2]);
        if !read? {
            return Err(BadInput("a mask picture was not read").into());
        }
    }
    Ok(())
}

/// `gfx::put_surface_pixel` (components, x, y).
pub unsafe fn put_pixel(components: u32, x: u32, y: u32) -> Result<(), Refused> {
    unsafe { crate::gfx::put_surface_pixel::put_surface_pixel(components, x, y)? };
    Ok(())
}

/// `d3d::lock_texture` (`lock`) or `d3d::unlock_texture` of the texture slot `slot` (their
/// flags unread).
pub unsafe fn lock(slot: u32, lock: bool) -> Result<(), Refused> {
    unsafe {
        if lock {
            crate::d3d::lock_texture::lock_texture(slot)?;
        } else {
            crate::d3d::unlock_texture::unlock_texture(slot)?;
        }
    }
    Ok(())
}

/// A mask read, copied to the 0x80000 block, the ground read, and the block `to` made
/// and drawn at rows 0x80 .. 0xFD.
unsafe fn mask(path: usize, to: &Dword, width: u32) -> Result<(), Refused> {
    unsafe {
        read((PICTURE_COPY.paths.addr() + path) as u32)?;
        std::ptr::copy(
            PICTURE.buffer.get() as *const u8,
            PICTURE_COPY.copy.get() as *mut u8,
            4 * COPIED as usize,
        );
        read(GROUND_NAMES.ground_path.get())?;
        for y in 0..ROWS {
            for x in 0..width.max(1) {
                let n = (y << 8).wrapping_add(x);
                let copy: u32 = PICTURE_COPY.copy.get();
                let mask = ((copy + 4 * n) as *const u32).read_unaligned();
                let pixel = (mask << 8) & 0xFF00_0000 | 0xFF_FFFF;
                let block: u32 = to.get();
                ((block + 4 * n) as *mut u32).write_unaligned(pixel);
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
pub unsafe fn load_mask_pictures() -> Result<(), Refused> {
    unsafe {
        let ground: i32 = GROUND_NAMES.ground.get() as i32;
        if ground >= NAMES {
            return Err(BadInput("a ground past the six names").into());
        }
        let mut name = GROUND_NAMES.names.addr() as u32;
        for _ in 0..ground.max(0) {
            while *(name as *const u8) != 0 {
                name += 1;
            }
            name += 1;
        }
        GROUND_NAMES.ground_path.set(name);
        for (bytes, list, to) in blocks() {
            to.set(crate::memory::allocate::allocate(list, bytes)? as u32);
        }
        lock(KIND_TEXTURES.kind_first_textures[202].bits(), true)?;
        let width: u32 = game_read(SURFACE_WIDTH_RVA);
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
        let [(_, _, mask_block), (_, _, bmask_block), _] = blocks();
        mask(MASK_PATH_AT, mask_block, width)?;
        mask(BMASK_PATH_AT, bmask_block, width)?;
        lock(KIND_TEXTURES.kind_first_textures[202].bits(), false)?;
        crate::scene::forget_slot_texture::forget_slot_texture(
            KIND_TEXTURES.kind_first_slots[202].bits(),
        )?;
        crate::memory::free_list::free_list(PICTURE_COPY.list.as_ptr())?;
        MASK_BLOCKS.seen.set(u32::MAX);
    }
    Ok(())
}
