//! `marks::load_mask_pictures` (0x4C8BD8): `init_marks`' step 10: the kind's two mask
//! pictures made 256 x 128 blocks of white with the mask as alpha, and drawn into a
//! texture surface.
//!
//! In:  nothing
//! Out: as `memory::free_list` leaves them (eax `dword_5D1E10`, ebx the list)
//!
//! The ground picture's name is the `dword_934CC8`-th of six (`files\m\snow1.pcx`,
//! `mud1`, `dust1`, `mud1`, `mud1`, `gumbo0`); it is read twice and never used (section
//! 16). The surface's lock and unlock are the game's (`sub_41AA4B` / `sub_41AB02`,
//! DirectDraw). One caller, `marks::init_marks`. Shadow mode runs the original alone (it
//! takes memory and reads files). Refuses a ground past the six names (before anything),
//! a surface wider than 256 (after the lock, unlocked again; the three blocks are left
//! taken), memory or a picture not had, as its steps do. Self-test: a redirected copy
//! whose calls go to stand-ins noting every register (blocks and a picture of the
//! test's), against the Rust version on the same stand-ins; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::race::level_init::Part;

const GROUND_RVA: usize = rva(0x934CC8); // g_marksGround: which of the names
const NAMES_RVA: usize = rva(0x934CD0); // "files\m\snow1.pcx" and five more
const NAMES: i32 = 6;
const GROUND_PATH_RVA: usize = rva(0x934CCC); // g_marksGroundPath
/// The blocks: bytes, their list, where the pointer goes.
const BLOCKS: [(u32, usize, usize); 3] = [
    (0x2_0000, rva(0x934D76), rva(0x934DB6)), // g_marksMaskBlock
    (0x2_0000, rva(0x934DBA), rva(0x934DFA)), // g_marksBMaskBlock
    (0x8_0000, rva(0x934E29), rva(0x934E69)), // g_marksPictureCopy
];
const COPY_LIST_RVA: usize = BLOCKS[2].1;
const COPY_RVA: usize = BLOCKS[2].2;
const SLOT_RVA: usize = rva(0x5D0E10); // dword_5D0E10: the surface drawn into
const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10: the texture slot forgotten
const NUMBERS_RVA: usize = rva(0x934EA8); // g_marksKindPictureNumbers: 0x22 dwords
const MOST_KIND: i32 = 0x21;
const MASK_PATH_RVA: usize = rva(0x934E6D); // "files\m\mask00.pcx"
const BMASK_PATH_RVA: usize = rva(0x934E80); // "files\m\bmask00.pcx"
/// The three paths' digits ("files\m\bdata00.pcx" the third's, not read here).
const DIGITS_RVA: [usize; 3] = [rva(0x934E79), rva(0x934E8D), rva(0x934EA1)];
const DONE_RVA: usize = rva(0x934D6E); // dword_934D6E: -1 at the end
const SURFACE_WIDTH_RVA: usize = rva(0x4F49A0); // g_surface.width
const MOST_WIDTH: u32 = 0x100;
const COPIED: u32 = 0x8000; // dwords
const ROWS: u32 = 0x7E;

/// Place 2 at 256 x 128 (the picture's sizes kept round it); refused when not read.
unsafe fn read(g: &mut Regs, path: u32, part: Part) -> Result<(), Refused> {
    use crate::gfx::picture::{HEIGHT_RVA, ROW_BYTES_RVA, WIDTH_RVA};
    unsafe {
        let sizes: [u32; 3] = [
            game_read(WIDTH_RVA),
            game_read(HEIGHT_RVA),
            game_read(ROW_BYTES_RVA),
        ];
        game_write(WIDTH_RVA, 0x100u32);
        game_write(HEIGHT_RVA, 0x80u32);
        game_write(ROW_BYTES_RVA, 0xFFFF_FFFEu32);
        g.eax = path;
        let read = part(2, g);
        game_write(WIDTH_RVA, sizes[0]);
        game_write(HEIGHT_RVA, sizes[1]);
        game_write(ROW_BYTES_RVA, sizes[2]);
        read
    }
}

/// A mask read, copied to the 0x80000 block, the ground read, and the block `to` made
/// and drawn at rows 0x80 .. 0xFD.
unsafe fn mask(
    g: &mut Regs,
    path: usize,
    to: usize,
    width: u32,
    part: Part,
) -> Result<(), Refused> {
    use crate::gfx::picture::PICTURE_RVA;
    unsafe {
        read(g, game_addr(path) as u32, part)?;
        g.esi = game_read(PICTURE_RVA);
        g.edi = game_read(COPY_RVA);
        for _ in 0..COPIED {
            g.eax = (g.esi as *const u32).read_unaligned();
            (g.edi as *mut u32).write_unaligned(g.eax);
            g.esi += 4;
            g.edi += 4;
        }
        read(g, game_read(GROUND_PATH_RVA), part)?;
        g.edx = 0;
        loop {
            g.ecx = 0;
            loop {
                g.ebp = (g.edx << 8).wrapping_add(g.ecx);
                g.esi = game_read(COPY_RVA);
                g.edi = game_read(PICTURE_RVA);
                let mask = ((g.esi + 4 * g.ebp) as *const u32).read_unaligned();
                g.ebx = 0xFF_FFFF;
                g.eax = (mask << 8) & 0xFF00_0000 | g.ebx;
                g.esi = game_read(to);
                ((g.esi + 4 * g.ebp) as *mut u32).write_unaligned(g.eax);
                let y = g.edx;
                g.edx += 0x80;
                part(3, g)?;
                g.edx = y;
                g.ecx += 1;
                if g.ecx >= width {
                    break;
                }
            }
            g.edx += 1;
            if g.edx >= ROWS {
                break;
            }
        }
    }
    Ok(())
}

/// The Rust version: the ground's name skipped to (`g_marksGroundPath`); the three blocks
/// taken (place 0); the surface locked (1); the kind (held to 0 .. 0x21) its picture
/// number's two digits in the three paths; each mask (reading place 2) into its block and
/// drawn (`put_surface_pixel`, 3); unlocked (4), the texture slot forgotten (5), the
/// 0x80000 block's list freed (6), `dword_934D6E` -1.
pub unsafe fn load_mask_pictures(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let ground: i32 = game_read(GROUND_RVA);
        if ground >= NAMES {
            return Err(BadInput("a ground past the six names").into());
        }
        g.esi = game_addr(NAMES_RVA) as u32;
        for _ in 0..ground.max(0) {
            while *(g.esi as *const u8) != 0 {
                g.esi += 1;
            }
            g.esi += 1;
        }
        game_write(GROUND_PATH_RVA, g.esi);
        for (bytes, list, to) in BLOCKS {
            g.eax = bytes;
            g.ebx = game_addr(list) as u32;
            part(0, &mut g)?;
            game_write(to, g.eax);
        }
        g.eax = game_read(SLOT_RVA);
        part(1, &mut g)?;
        let width: u32 = game_read(SURFACE_WIDTH_RVA);
        if width > MOST_WIDTH {
            g.eax = game_read(SLOT_RVA);
            part(4, &mut g)?;
            return Err(BadInput("a mask surface wider than 256").into());
        }
        let kind = (game_read::<u32>(crate::marks::kind_values::KIND_RVA) as i32)
            .clamp(0, MOST_KIND) as u32;
        let number: u32 = game_read(NUMBERS_RVA + 4 * kind as usize);
        let (tens, ones) = (number / 10, number % 10);
        g.eax = (tens & !0xFF) | (tens as u8).wrapping_add(b'0') as u32;
        g.edx = ones + b'0' as u32;
        g.ebx = 10;
        for d in DIGITS_RVA {
            game_write(d, [g.eax as u8, g.edx as u8]);
        }
        mask(&mut g, MASK_PATH_RVA, BLOCKS[0].2, width, part)?;
        mask(&mut g, BMASK_PATH_RVA, BLOCKS[1].2, width, part)?;
        g.eax = game_read(SLOT_RVA);
        part(4, &mut g)?;
        g.eax = game_read(TEXTURE_RVA);
        part(5, &mut g)?;
        g.ebx = game_addr(COPY_LIST_RVA) as u32;
        part(6, &mut g)?;
        game_write(DONE_RVA, u32::MAX);
    }
    Ok(g)
}

/// In the game: 0 `memory::allocate`, 1 and 4 the surface's lock and unlock (the game's),
/// 2 `gfx::read_picture_file` (refused when it fails: the original then counts 0x1FFFFFFF
/// down and leaves with its stack one off), 3 `gfx::put_surface_pixel`,
/// 5 `scene::forget_slot_texture`, 6 `memory::free_list`.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => crate::race::level_init::game_part(0, r)?,
            1 => crate::calls::call(rva(0x41AA4B), r),
            2 => {
                if !crate::gfx::read_picture_file::read_picture_file(r.eax as *mut u8)? {
                    return Err(BadInput("a mask picture was not read").into());
                }
                r.set_carry(false);
            }
            3 => r.eax = crate::gfx::put_surface_pixel::put_surface_pixel(r)?.eax,
            4 => crate::calls::call(rva(0x41AB02), r),
            5 => crate::scene::forget_slot_texture::forget_slot_texture(r.eax)?,
            _ => crate::memory::free_list::free_list(r.ebx as *mut crate::memory::MemoryList)?,
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { load_mask_pictures(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
