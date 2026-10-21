//! The system textures' lock and unlock (`d3d::lock_texture`, `d3d::unlock_texture`,
//! `d3d::texture_surface_info`): what they share, and their test. A system texture's
//! record (`unk_5B521C`, 0x78 each, `platform_3d`'s `TextureRecord`): +4 width, +8
//! height, +0xC its pixel kind's description (+4 bytes a pixel, +8 the format), +0x14 its
//! first level's surface (a DirectDraw surface: `Lock` at +0x64 and `Unlock` at +0x80 of
//! its method table), +0x60 locked. The surface's methods are called through its own
//! table, so the same code locks the game's DirectDraw surfaces and the standalone's
//! (`platform_3d`'s objects).

use crate::errors::FatalError;
use crate::framework::{Regs, SelfTest, game_addr, rva};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;

pub const RECORDS_RVA: usize = rva(0x5B521C); // unk_5B521C
pub const RECORD: u32 = 0x78;
pub const DESCRIPTION_RVA: usize = rva(0x5ABA18); // dword_5ABA18: a DDSURFACEDESC
pub const DESCRIPTION_LEN: usize = 0x6C;
pub const SURFACE_RVA: usize = rva(0x4F4998); // g_surface (0x48)

pub fn record(slot: u32) -> u32 {
    (game_addr(RECORDS_RVA) as u32).wrapping_add(slot.wrapping_mul(RECORD))
}

pub fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

pub fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// A surface's method at `offset` of its table, called with `args` after `this`
/// (`stdcall`).
pub unsafe fn method(surface: u32, offset: u32, args: &[u32]) -> i32 {
    unsafe {
        let table = get(surface);
        let f = get(table + offset);
        match args {
            [a] => {
                let f: unsafe extern "system" fn(u32, u32) -> i32 = std::mem::transmute(f as usize);
                f(surface, *a)
            }
            [a, b, c, d] => {
                let f: unsafe extern "system" fn(u32, u32, u32, u32, u32) -> i32 =
                    std::mem::transmute(f as usize);
                f(surface, *a, *b, *c, *d)
            }
            _ => unreachable!("the two methods used here"),
        }
    }
}

pub const fn fatal(code: u32) -> FatalError {
    FatalError { code, message: "" }
}

pub use rc2k_core::d3d::textures::compared_with_one;

// ---- the test: a surface of the test's ----

/// Each method call: (offset, this, the arguments).
static CALLS: Mutex<Vec<(u32, [u32; 5])>> = Mutex::new(Vec::new());
static mut ANSWER: i32 = 0;
static mut PIXELS: [u8; 0x400] = [0; 0x400];
static mut TABLE: [u32; 0x40] = [0; 0x40];
static mut OBJECT: [u32; 2] = [0, 7];
/// The pixel kind's description of the test: bytes a pixel 2, the format after.
static mut KIND: [u32; 8] = [0, 2, 0x55AA, 0, 0, 0, 0, 0];

unsafe extern "system" fn test_lock(this: u32, a: u32, description: u32, c: u32, d: u32) -> i32 {
    CALLS
        .lock()
        .unwrap()
        .push((0x64, [this, a, description, c, d]));
    unsafe {
        if ANSWER == 0 {
            put(description + 0x24, (&raw const PIXELS) as u32);
            put(description + 0x10, 0x80);
        }
        ANSWER
    }
}

unsafe extern "system" fn test_unlock(this: u32, a: u32) -> i32 {
    CALLS.lock().unwrap().push((0x80, [this, a, 0, 0, 0]));
    unsafe { ANSWER }
}

pub fn test_surface() -> u32 {
    unsafe {
        let table = &raw mut TABLE;
        (*table)[0x64 / 4] = test_lock as *const () as u32;
        (*table)[0x80 / 4] = test_unlock as *const () as u32;
        (&raw mut OBJECT).cast::<u32>().write(table as u32);
        (&raw const OBJECT) as u32
    }
}

/// The test of one of the two: for each (surface or none, locked, width), the original
/// (`call`, eax the slot) against `rust` on the last slot of the test's: the method calls,
/// the flags, the registers, the record, the description, `g_surface` and the clip.
pub fn test_slot(
    mut t: SelfTest,
    call: crate::screens::harness::CallOriginal,
    cases: &[(bool, u32, u32)],
    rust: &dyn Fn(&Regs) -> Result<Regs, crate::errors::Refused>,
) -> (usize, usize) {
    use crate::gfx::surface::{CLIP_BOTTOM_RVA, CLIP_LEFT_RVA, CLIP_RIGHT_RVA, CLIP_TOP_RVA};
    let slot = 0x13F;
    let regions: Vec<Region> = vec![
        (record(slot), RECORD as usize),
        (game_addr(DESCRIPTION_RVA) as u32, DESCRIPTION_LEN),
        (game_addr(SURFACE_RVA) as u32, 0x48),
        at(CLIP_LEFT_RVA),
        at(CLIP_TOP_RVA),
        at(CLIP_RIGHT_RVA),
        at(CLIP_BOTTOM_RVA),
    ];
    let saved = capture(&regions);
    let surface = test_surface();
    for &(with_surface, locked, width) in cases {
        let prepare = || {
            restore(&regions, &saved);
            let r = record(slot);
            put(r + 4, width);
            put(r + 8, 0x40);
            put(r + 0xC, (&raw const KIND) as u32);
            put(r + 0x14, if with_surface { surface } else { 0 });
            put(r + 0x60, locked);
            CALLS.lock().unwrap().clear();
        };
        let before = Regs {
            eax: slot,
            ..Regs::SENTINEL
        };
        let take = || std::mem::take(&mut *CALLS.lock().unwrap());
        prepare();
        let mut ro = before;
        unsafe { call(&mut ro) };
        let theirs = (
            take(),
            ro.carry(),
            ro.eflags & 0x40 != 0,
            ro.preserved_except(&before, 0),
            capture(&regions),
        );
        prepare();
        let mine = rust(&before);
        let mine = match mine {
            Ok(m) => (
                take(),
                m.carry(),
                m.eflags & 0x40 != 0,
                m.preserved_except(&before, 0),
                capture(&regions),
            ),
            Err(_) => (take(), false, false, false, Vec::new()),
        };
        t.check(theirs == mine, || {
            format!(
                "surface {with_surface}, locked {locked}, width {width:#x}: calls {:x?} / {:x?}, carry {} / {}, zero {} / {}, kept {} / {}, state same {}",
                theirs.0, mine.0, theirs.1, mine.1, theirs.2, mine.2, theirs.3, mine.3, theirs.4 == mine.4
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
