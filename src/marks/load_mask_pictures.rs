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
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::scene::kind_texture_store::KIND_TEXTURES;

const GROUND_RVA: Slot = Slot::at(0x934CC8); // g_marksGround: which of the names
const NAMES_RVA: Slot = Slot::at(0x934CD0); // "files\m\snow1.pcx" and five more
const NAMES: i32 = 6;
const GROUND_PATH_RVA: Slot = Slot::at(0x934CCC); // g_marksGroundPath
/// The blocks: bytes, their list, where the pointer goes.
const BLOCKS: [(u32, Slot, Slot); 3] = [
    (0x2_0000, Slot::at(0x934D76), Slot::at(0x934DB6)), // g_marksMaskBlock
    (0x2_0000, Slot::at(0x934DBA), Slot::at(0x934DFA)), // g_marksBMaskBlock
    (0x8_0000, Slot::at(0x934E29), Slot::at(0x934E69)), // g_marksPictureCopy
];
const COPY_LIST_RVA: Slot = BLOCKS[2].1;
const COPY_RVA: Slot = BLOCKS[2].2;
const NUMBERS_RVA: Slot = Slot::at(0x934EA8); // g_marksKindPictureNumbers: 0x22 dwords
const MOST_KIND: i32 = 0x21;
const MASK_PATH_RVA: Slot = Slot::at(0x934E6D); // "files\m\mask00.pcx"
const BMASK_PATH_RVA: Slot = Slot::at(0x934E80); // "files\m\bmask00.pcx"
/// The three paths' digits ("files\m\bdata00.pcx" the third's, not read here).
const DIGITS_RVA: [Slot; 3] = [Slot::at(0x934E79), Slot::at(0x934E8D), Slot::at(0x934EA1)];
const DONE_RVA: Slot = Slot::at(0x934D6E); // dword_934D6E: -1 at the end
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
unsafe fn mask(path: Slot, to: Slot, width: u32) -> Result<(), Refused> {
    unsafe {
        read(game_addr(path) as u32)?;
        std::ptr::copy(
            PICTURE.buffer.get() as *const u8,
            game_read::<u32>(COPY_RVA) as *mut u8,
            4 * COPIED as usize,
        );
        read(game_read(GROUND_PATH_RVA))?;
        for y in 0..ROWS {
            for x in 0..width.max(1) {
                let n = (y << 8).wrapping_add(x);
                let copy: u32 = game_read(COPY_RVA);
                let mask = ((copy + 4 * n) as *const u32).read_unaligned();
                let pixel = (mask << 8) & 0xFF00_0000 | 0xFF_FFFF;
                let block: u32 = game_read(to);
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
        let ground: i32 = game_read(GROUND_RVA);
        if ground >= NAMES {
            return Err(BadInput("a ground past the six names").into());
        }
        let mut name = game_addr(NAMES_RVA) as u32;
        for _ in 0..ground.max(0) {
            while *(name as *const u8) != 0 {
                name += 1;
            }
            name += 1;
        }
        game_write(GROUND_PATH_RVA, name);
        for (bytes, list, to) in BLOCKS {
            game_write(to, crate::marks::blocks::take(list, bytes)?);
        }
        lock(KIND_TEXTURES.kind_first_textures[202].bits(), true)?;
        let width: u32 = game_read(SURFACE_WIDTH_RVA);
        if width > MOST_WIDTH {
            lock(KIND_TEXTURES.kind_first_textures[202].bits(), false)?;
            return Err(BadInput("a mask surface wider than 256").into());
        }
        let kind = (game_read::<u32>(crate::marks::kind_values::KIND_RVA) as i32)
            .clamp(0, MOST_KIND) as u32;
        let number: u32 = game_read(NUMBERS_RVA + 4 * kind as usize);
        let digits = [
            ((number / 10) as u8).wrapping_add(b'0'),
            (number % 10) as u8 + b'0',
        ];
        for d in DIGITS_RVA {
            game_write(d, digits);
        }
        mask(MASK_PATH_RVA, BLOCKS[0].2, width)?;
        mask(BMASK_PATH_RVA, BLOCKS[1].2, width)?;
        lock(KIND_TEXTURES.kind_first_textures[202].bits(), false)?;
        crate::scene::forget_slot_texture::forget_slot_texture(
            KIND_TEXTURES.kind_first_slots[202].bits(),
        )?;
        crate::memory::free_list::free_list(
            game_addr(COPY_LIST_RVA) as *mut crate::memory::MemoryList
        )?;
        game_write(DONE_RVA, u32::MAX);
    }
    Ok(())
}
