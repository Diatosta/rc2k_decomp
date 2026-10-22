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

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::masks::NOTED;
use crate::race::level_init::Part;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::load_mask_pictures", rva(0x4C8BD8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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
const PATHS_LEN: usize = 0x3B;
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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { load_mask_pictures(r, &mut game_part) } {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The test's blocks, given out in turn by the allocation stand-in.
pub(crate) static BLOCK_AT: [AtomicU32; 3] =
    [AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0)];
pub(crate) static TAKEN: AtomicUsize = AtomicUsize::new(0);
/// The next picture read fails (once).
pub(crate) static FAIL_READ: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// The stand-ins: each call noted; the allocation gives the next block, it and the
/// picture read with the carry clear (set once after `FAIL_READ`).
pub(crate) fn note(place: u32, r: &mut Regs) {
    NOTED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]));
    if place == 0 {
        r.eax = BLOCK_AT[TAKEN.fetch_add(1, Ordering::SeqCst) % 3].load(Ordering::SeqCst);
    }
    if place == 0 || place == 2 {
        r.set_carry(place == 2 && FAIL_READ.swap(false, Ordering::SeqCst));
    }
}

pub(crate) fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::gfx::picture::{HEIGHT_RVA, PICTURE_RVA, ROW_BYTES_RVA, WIDTH_RVA};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // lea esi, aFilesMSnow1Pcx
    let entry: &[u8] = &[0x8D, 0x35, 0xD0, 0x4C, 0x93, 0x00];
    let copy = match crate::race::mud::copy(
        0x4C8BD8,
        0x375,
        entry,
        &[
            0x30, 0x53, 0x76, 0x93, 0x11D, 0x19D, 0x204, 0x24F, 0x2CF, 0x336, 0x350, 0x35A, 0x365,
        ],
        &[
            (0x4028A7, 0),
            (0x41AA4B, 1),
            (0x4119F4, 2),
            (0x4065BB, 3),
            (0x41AB02, 4),
            (0x44EB66, 5),
            (0x4028F5, 6),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note);
    let mut blocks: [Vec<u32>; 3] = [vec![0; 0x8000], vec![0; 0x8000], vec![0; 0x2_0000]];
    for (b, at) in blocks.iter_mut().zip(&BLOCK_AT) {
        at.store(b.as_mut_ptr() as u32, Ordering::SeqCst);
    }
    let picture: Vec<u32> = (0..COPIED)
        .map(|n| n.wrapping_mul(0x0103_0507) ^ 0x5A00_C3A5)
        .collect();
    let a = |r: usize| game_addr(r) as u32;
    let mut regions: Vec<Region> = [
        GROUND_RVA,
        GROUND_PATH_RVA,
        BLOCKS[0].2,
        BLOCKS[1].2,
        COPY_RVA,
        DONE_RVA,
        SURFACE_WIDTH_RVA,
        PICTURE_RVA,
        WIDTH_RVA,
        HEIGHT_RVA,
        ROW_BYTES_RVA,
        crate::marks::kind_values::KIND_RVA,
    ]
    .map(at)
    .to_vec();
    regions.push((a(MASK_PATH_RVA), PATHS_LEN));
    let saved = capture(&regions);
    let mut blocks_of = regions.clone();
    blocks_of.extend(blocks.iter().map(|b| (b.as_ptr() as u32, 4 * b.len())));
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    let cases: [(i32, i32, u32); 6] = [
        (0, 0, 0x100),
        (7, 2, 3),
        (0x21, 5, 1),
        (-1, -1, 0),
        (0x100, 0, 2),
        (12, 3, 0x40),
    ];
    for (kind, ground, width) in cases {
        let prepare = |blocks: &mut [Vec<u32>; 3]| unsafe {
            restore(&regions, &saved);
            game_write(crate::marks::kind_values::KIND_RVA, kind);
            game_write(GROUND_RVA, ground);
            game_write(SURFACE_WIDTH_RVA, width);
            game_write(PICTURE_RVA, picture.as_ptr() as u32);
            for b in blocks.iter_mut() {
                b.fill(0xAAAA_AAAA);
            }
            TAKEN.store(0, Ordering::SeqCst);
            NOTED.lock().unwrap().clear();
        };
        prepare(&mut blocks);
        let mut theirs = Regs::SENTINEL;
        unsafe { crate::framework::call_game(&mut theirs, copy.addr()) };
        let theirs = (
            regs(&theirs),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&blocks_of),
        );
        prepare(&mut blocks);
        let mine = unsafe { load_mask_pictures(&Regs::SENTINEL, &mut test_part) };
        let mine = (
            mine.as_ref().map(regs).unwrap_or_default(),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&blocks_of),
        );
        t.check(theirs == mine, || {
            let first = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
            format!(
                "kind {kind:#x}, ground {ground}, width {width:#x}: original {:x?}, {} calls; \
                 rust {:x?}, {} calls; first differing call {first:?}; memory same {}",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len(),
                theirs.2 == mine.2
            )
        });
    }
    // The refusals.
    for (ground, width) in [(6, 0x100), (0, 0x101)] {
        unsafe {
            restore(&regions, &saved);
            game_write(GROUND_RVA, ground);
            game_write(SURFACE_WIDTH_RVA, width);
            TAKEN.store(0, Ordering::SeqCst);
            NOTED.lock().unwrap().clear();
        }
        let got = unsafe { load_mask_pictures(&Regs::SENTINEL, &mut test_part) };
        let places: Vec<u32> = NOTED.lock().unwrap().drain(..).map(|(p, _)| p).collect();
        let want: &[u32] = if ground == 6 { &[] } else { &[0, 0, 0, 1, 4] };
        t.check(got.is_err() && places == want, || {
            format!(
                "ground {ground}, width {width:#x}: refused {}, calls {places:?}",
                got.is_err()
            )
        });
    }
    restore(&regions, &saved);
    drop(blocks);
    t.finish()
}
