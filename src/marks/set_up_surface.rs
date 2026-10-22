//! `marks::set_up_surface` (0x4CB0BC): `init_marks`' step 12: the surface picture
//! (`files\m\surfdata.pcx`, its alpha the inverted third byte of `files\m\surfmask.pcx`)
//! drawn into a 256 x 256 texture surface; a ring of 0x400 records of 0x94 bytes; two
//! tables of texture places (0x40 cells of an 8 x 8 grid, and 0x40 of 4 x 4 sub-cells).
//!
//! In:  nothing
//! Out: eax 0, ebx 0, ecx the ring, esi 0xF0, edi and ebp past the 0x40000 block, edx
//!      as the steps leave it
//!
//! The surface's lock and unlock are the game's (`sub_41AA4B` / `sub_41AB02`, DirectDraw).
//! Callers: `marks::init_marks` and code at 0x4D1BC8 (not in a function). Shadow mode runs
//! the original alone (it takes memory and reads files). Refuses (section 15) memory not
//! had; a surface under 256 x 256 or a picture not read, after the lock (unlocked again,
//! the 0x40000 block left taken). Self-test: a redirected copy whose calls go to
//! stand-ins noting every register (blocks and a picture of the test's), against the Rust
//! version on the same stand-ins, with the game's constants and two other sets; put back
//! afterwards.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::load_mask_pictures::{BLOCK_AT, FAIL_READ, TAKEN, note, test_part};
use crate::marks::masks::NOTED;
use crate::math::x87;
use crate::race::level_init::Part;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::set_up_surface", rva(0x4CB0BC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const MASK_LIST_RVA: usize = rva(0x93A1DA); // stru_93A1DA
const MASK_BYTES: u32 = 0x4_0000;
const SLOT_RVA: usize = rva(0x5D0E34); // dword_5D0E34: the surface drawn into
const TEXTURE_RVA: usize = rva(0x5D1E34); // dword_5D1E34: the texture slot forgotten
const SURFMASK_RVA: usize = rva(0x935751); // "files\m\surfmask.pcx"
const SURFDATA_RVA: usize = rva(0x93573C); // "files\m\surfdata.pcx"
const PIXELS: u32 = 0x1_0000;
const SURFACE_WIDTH_RVA: usize = rva(0x4F49A0); // g_surface.width
const SURFACE_HEIGHT_RVA: usize = rva(0x4F49A4); // g_surface.height
const SIDE: u32 = 0x100;
const RING_LIST_RVA: usize = rva(0x935620); // stru_935620
const RING_BYTES: u32 = 0x2_5000;
pub const RING_RVA: usize = rva(0x935668); // g_marksRingBlock (g_marksRing is 0x93A5C8)
pub const RING_CURSOR_RVA: usize = rva(0x935660); // g_marksRingCursor
pub const RING_COUNT_RVA: usize = rva(0x9356C8); // g_marksRingCount
const RING: u32 = 0x400;
const RECORD: u32 = 0x94;
const CELLS_RVA: usize = rva(0x935766); // g_marksCellPlaces: 0x40 of four floats
const SUBCELLS_RVA: usize = rva(0x935B66); // g_marksSubcellPlaces: 0x40 of 0x100 bytes
const CELLS: u32 = 0x40;
/// flt_7651C0 (1/8), flt_7651C8 (1/512), flt_7651CC (1/256), flt_7651D0 (1/32).
const EIGHTH_RVA: usize = rva(0x7651C0);
const EDGE_RVA: usize = rva(0x7651C8);
const PIXEL_RVA: usize = rva(0x7651CC);
const STEP_RVA: usize = rva(0x7651D0);

/// Place 2 at 256 x 256 (the picture's sizes kept round it); the carry set when not read.
unsafe fn read(g: &mut Regs, path: usize, part: Part) -> Result<bool, Refused> {
    use crate::gfx::picture::{HEIGHT_RVA, ROW_BYTES_RVA, WIDTH_RVA};
    unsafe {
        let sizes: [u32; 3] = [
            game_read(WIDTH_RVA),
            game_read(HEIGHT_RVA),
            game_read(ROW_BYTES_RVA),
        ];
        game_write(WIDTH_RVA, SIDE);
        game_write(HEIGHT_RVA, SIDE);
        game_write(ROW_BYTES_RVA, 0xFFFF_FFFEu32);
        g.eax = game_addr(path) as u32;
        let read = part(2, g);
        game_write(WIDTH_RVA, sizes[0]);
        game_write(HEIGHT_RVA, sizes[1]);
        game_write(ROW_BYTES_RVA, sizes[2]);
        read?;
        Ok(!g.carry())
    }
}

/// Unlocks the surface (place 4) and refuses with `why`.
unsafe fn unlock_refusing(g: &mut Regs, part: Part, why: &'static str) -> Result<Regs, Refused> {
    unsafe {
        g.eax = game_read(SLOT_RVA);
        part(4, g)?;
    }
    Err(BadInput(why).into())
}

/// The Rust version (see the header). Places: 0 `memory::allocate`, 1 / 4 the lock /
/// unlock, 2 `gfx::read_picture_file`, 3 `gfx::put_surface_pixel`,
/// 5 `scene::forget_slot_texture`, 6 `memory::free_list`, 7 `nullsub_33`.
pub unsafe fn set_up_surface(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    use crate::gfx::picture::PICTURE_RVA;
    let mut g = *entry;
    unsafe {
        g.eax = MASK_BYTES;
        g.ebx = game_addr(MASK_LIST_RVA) as u32;
        part(0, &mut g)?;
        let block = g.eax;
        g.eax = game_read(SLOT_RVA);
        part(1, &mut g)?;
        if game_read::<u32>(SURFACE_WIDTH_RVA) < SIDE || game_read::<u32>(SURFACE_HEIGHT_RVA) < SIDE
        {
            return unlock_refusing(&mut g, part, "a surface under 256 x 256");
        }
        if !read(&mut g, SURFMASK_RVA, part)? {
            return unlock_refusing(&mut g, part, "surfmask.pcx was not read");
        }
        g.esi = game_read(PICTURE_RVA);
        g.edi = block;
        for _ in 0..PIXELS {
            g.eax = (g.esi as *const u32).read_unaligned();
            (g.edi as *mut u32).write_unaligned(g.eax);
            g.esi += 4;
            g.edi += 4;
        }
        g.eax = PIXELS;
        if !read(&mut g, SURFDATA_RVA, part)? {
            return unlock_refusing(&mut g, part, "surfdata.pcx was not read");
        }
        g.esi = game_read(PICTURE_RVA);
        g.ebp = block;
        for n in 0..PIXELS {
            g.ecx = n & 0xFF;
            g.edx = n >> 8;
            g.eax = (g.esi as *const u32).read_unaligned();
            g.ebx = (g.ebp as *const u32).read_unaligned();
            g.ebx = ((g.ebx << 8) & 0xFF00_0000) ^ 0xFF00_0000;
            g.eax = (g.eax & 0xFF_FFFF) | g.ebx;
            let kept = g;
            part(3, &mut g)?;
            g = Regs {
                eflags: g.eflags,
                ..kept
            };
            g.esi += 4;
            g.ebp += 4;
        }
        g.eax = game_read(SLOT_RVA);
        part(4, &mut g)?;
        g.eax = game_read(TEXTURE_RVA);
        part(5, &mut g)?;
        g.eax = block;
        g.ebx = game_addr(MASK_LIST_RVA) as u32;
        part(6, &mut g)?;

        g.eax = RING_BYTES;
        g.ebx = game_addr(RING_LIST_RVA) as u32;
        part(0, &mut g)?;
        game_write(RING_RVA, g.eax);
        game_write(RING_CURSOR_RVA, g.eax);
        game_write(RING_COUNT_RVA, 0u32);
        g.esi = game_read(RING_RVA);
        for i in 0..RING {
            g.ebx = (i.wrapping_sub(1) & (RING - 1)) * RECORD + g.esi;
            g.ecx = ((i + 1) & (RING - 1)) * RECORD + g.esi;
            g.eax = i * RECORD;
            ((g.eax + g.esi) as *mut u32).write_unaligned(g.ebx);
            ((g.eax + g.esi + 4) as *mut u32).write_unaligned(g.ecx);
        }
        g.eax = RING;

        let [eighth, edge, pixel, step]: [u32; 4] =
            [EIGHTH_RVA, EDGE_RVA, PIXEL_RVA, STEP_RVA].map(|r| game_read(r));
        let put = |at: u32, v: u32| (at as *mut u32).write_unaligned(v);
        let get = |at: u32| (at as *const u32).read_unaligned();
        let cells = game_addr(CELLS_RVA) as u32;
        for k in (0..CELLS).rev() {
            g.esi = k * 0x10;
            g.eax = k & 7;
            g.ebx = k >> 3;
            let (u0, u1) = x87::int_times_plus_on(g.eax as i32, eighth, edge, eighth, pixel);
            put(cells + g.esi, u0);
            put(cells + g.esi + 8, u1);
            let (v0, v1) = x87::int_times_plus_on(g.ebx as i32, eighth, edge, eighth, pixel);
            put(cells + g.esi + 4, v0);
            put(cells + g.esi + 0xC, v1);
        }
        // Each cell's 4 rows of 4 sub-cells: 0x100 bytes.
        let sub = game_addr(SUBCELLS_RVA) as u32;
        for j in (0..CELLS).rev() {
            g.esi = j * 0x100;
            g.eax = j & 7;
            g.ebx = j >> 3;
            let (u0, u1) = x87::int_times_plus_on(g.eax as i32, eighth, edge, step, pixel);
            put(sub + g.esi, u0);
            put(sub + g.esi + 8, u1);
            let (v0, v1) = x87::int_times_plus_on(g.ebx as i32, eighth, edge, step, pixel);
            put(sub + g.esi + 4, v0);
            put(sub + g.esi + 0xC, v1);
            let across = |g: &mut Regs| {
                let p = sub + g.esi;
                put(p + 0x10, x87::add(get(p), step));
                put(p + 0x14, x87::loaded(get(p + 4)));
                put(p + 0x18, x87::add(get(p + 8), step));
                put(p + 0x1C, x87::loaded(get(p + 0xC)));
                g.esi += 0x10;
            };
            for _ in 0..3 {
                across(&mut g);
            }
            for _ in 0..3 {
                let p = sub + g.esi;
                put(p + 0x10, x87::sub_thrice(get(p), step));
                put(p + 0x14, x87::add(get(p + 4), step));
                put(p + 0x18, x87::sub_thrice(get(p + 8), step));
                put(p + 0x1C, x87::add(get(p + 0xC), step));
                g.esi += 0x10;
                for _ in 0..3 {
                    across(&mut g);
                }
            }
        }
        part(7, &mut g)?;
    }
    Ok(g)
}

/// In the game (see `set_up_surface`).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => crate::race::level_init::game_part(0, r)?,
            1 => crate::calls::call(rva(0x41AA4B), r),
            2 => {
                let read = crate::gfx::read_picture_file::read_picture_file(r.eax as *mut u8)?;
                r.set_carry(!read);
            }
            3 => r.eax = crate::gfx::put_surface_pixel::put_surface_pixel(r)?.eax,
            4 => crate::calls::call(rva(0x41AB02), r),
            5 => crate::scene::forget_slot_texture::forget_slot_texture(r.eax)?,
            6 => crate::memory::free_list::free_list(r.ebx as *mut crate::memory::MemoryList)?,
            // `nullsub_33`.
            _ => {}
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
    match unsafe { set_up_surface(r, &mut game_part) } {
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

fn self_test() -> (usize, usize) {
    use crate::gfx::picture::{HEIGHT_RVA, PICTURE_RVA, ROW_BYTES_RVA, WIDTH_RVA};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // push 40000h
    let entry: &[u8] = &[0x68, 0x00, 0x00, 0x04, 0x00];
    let copy = match crate::race::mud::copy(
        0x4CB0BC,
        0x4EC,
        entry,
        &[
            0xC, 0x30, 0x6B, 0xE4, 0x146, 0x167, 0x171, 0x178, 0x188, 0x4E6,
        ],
        &[
            (0x4028A7, 0),
            (0x41AA4B, 1),
            (0x4119F4, 2),
            (0x4065BB, 3),
            (0x41AB02, 4),
            (0x44EB66, 5),
            (0x4028F5, 6),
            (0x4D0BA7, 7),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note);
    let mut blocks: [Vec<u32>; 2] = [
        vec![0; (MASK_BYTES / 4) as usize],
        vec![0; (RING_BYTES / 4) as usize],
    ];
    for (b, at) in blocks.iter_mut().zip(&BLOCK_AT) {
        at.store(b.as_mut_ptr() as u32, Ordering::SeqCst);
    }
    let picture: Vec<u32> = (0..PIXELS)
        .map(|n| n.wrapping_mul(0x0103_0507) ^ 0x5A00_C3A5)
        .collect();
    let a = |r: usize| game_addr(r) as u32;
    let mut regions: Vec<Region> = [
        RING_RVA,
        RING_CURSOR_RVA,
        RING_COUNT_RVA,
        SURFACE_WIDTH_RVA,
        SURFACE_HEIGHT_RVA,
        PICTURE_RVA,
        WIDTH_RVA,
        HEIGHT_RVA,
        ROW_BYTES_RVA,
    ]
    .map(at)
    .to_vec();
    regions.push((a(EIGHTH_RVA), 0x14));
    regions.push((a(CELLS_RVA), (CELLS * 0x10) as usize));
    regions.push((a(SUBCELLS_RVA), (CELLS * 0x100) as usize));
    let saved = capture(&regions);
    let mut blocks_of = regions.clone();
    blocks_of.extend(blocks.iter().map(|b| (b.as_ptr() as u32, 4 * b.len())));
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    let constants: [Option<[f32; 4]>; 3] = [
        None,
        Some([0.1, 1.0 / 3.0, 0.7, 1.0 / 7.0]),
        Some([-3.5e7, 1.1e-3, 9.9, 2.0e-9]),
    ];
    for c in constants {
        let prepare = |blocks: &mut [Vec<u32>; 2]| unsafe {
            restore(&regions, &saved);
            if let Some(c) = c {
                for (r, v) in [EIGHTH_RVA, EDGE_RVA, PIXEL_RVA, STEP_RVA].iter().zip(c) {
                    game_write(*r, v.to_bits());
                }
            }
            game_write(SURFACE_WIDTH_RVA, SIDE);
            game_write(SURFACE_HEIGHT_RVA, SIDE);
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
        let mine = unsafe { set_up_surface(&Regs::SENTINEL, &mut test_part) };
        let mine = (
            mine.as_ref().map(regs).unwrap_or_default(),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&blocks_of),
        );
        t.check(theirs == mine, || {
            let first = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
            let memory: Vec<usize> = (0..theirs.2.len())
                .filter(|&i| theirs.2[i] != mine.2[i])
                .collect();
            format!(
                "constants {c:?}: original {:x?}, {} calls; rust {:x?}, {} calls; first \
                 differing call {first:?}; differing regions {memory:?}",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len(),
            )
        });
    }
    // The refusals: a narrow and a short surface, either picture not read.
    for (width, height, fail_at) in [
        (0xFF, SIDE, 0),
        (SIDE, 0xFF, 0),
        (SIDE, SIDE, 1),
        (SIDE, SIDE, 2),
    ] {
        unsafe {
            restore(&regions, &saved);
            game_write(SURFACE_WIDTH_RVA, width);
            game_write(SURFACE_HEIGHT_RVA, height);
            game_write(PICTURE_RVA, picture.as_ptr() as u32);
            TAKEN.store(0, Ordering::SeqCst);
            NOTED.lock().unwrap().clear();
        }
        // The `fail_at`-th read fails (0: none).
        let mut reads = 0;
        let mut part = |place: u32, r: &mut Regs| {
            if place == 2 {
                reads += 1;
                FAIL_READ.store(reads == fail_at, Ordering::SeqCst);
            }
            test_part(place, r)
        };
        let got = unsafe { set_up_surface(&Regs::SENTINEL, &mut part) };
        FAIL_READ.store(false, Ordering::SeqCst);
        let places: Vec<u32> = NOTED
            .lock()
            .unwrap()
            .drain(..)
            .map(|(p, _)| p)
            .filter(|&p| p != 3)
            .collect();
        let want: &[u32] = match fail_at {
            0 => &[0, 1, 4],
            1 => &[0, 1, 2, 4],
            _ => &[0, 1, 2, 2, 4],
        };
        t.check(got.is_err() && places == want, || {
            format!(
                "{width:#x} x {height:#x}, read {fail_at} failing: refused {}, calls {places:?}",
                got.is_err()
            )
        });
    }
    restore(&regions, &saved);
    drop(blocks);
    t.finish()
}
