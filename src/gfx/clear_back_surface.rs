//! `gfx::clear_back_surface` (0x40579A): with a front surface (`g_frontSurface`), the
//! screen locked (`gfx::lock_surface`; failing, nothing), its rows (`g_screenHeight` of
//! `g_screenRowDwords` dwords from `dword_4F4988`, `dword_4F498C` bytes between them)
//! zeroed and unlocked (`gfx::unlock_surface`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::viewer_frame` (at the replay's end), `gfx::set_screen_mode`,
//! `sub_405B10`. Its calls through `part`: 1 the lock (carry), 2 the unlock; in the game the
//! platform's through the game. A height of 0 is refused (the original goes round 2^32
//! times), before anything. Shadow mode runs the original alone (it locks the screen).
//! Self-test: a copy whose calls go to stand-ins, against the Rust version, the screen's
//! variables pointed at a buffer of the test's: no front surface, the lock failing, two
//! layouts.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{g, p};
use crate::race::viewer::{Part, Saved, answer, dispatch_parts, dword, run_both, test_part};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::clear_back_surface", rva(0x40579A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FRONT: usize = 0x4F32B4; // g_frontSurface
const PIXELS: usize = 0x4F4988; // dword_4F4988
const HEIGHT: usize = 0x4F4950; // g_screenHeight
const ROW_DWORDS: usize = 0x4F4960; // g_screenRowDwords
const GAP: usize = 0x4F498C; // dword_4F498C

/// The Rust version: the registers it leaves (the entry's).
pub fn clear_back_surface(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    if g(FRONT) == 0 {
        return Ok(*entry);
    }
    if g(HEIGHT) == 0 {
        return Err(BadInput("a screen of no rows: the original goes round 2^32 times").into());
    }
    let mut r = *entry;
    part(1, &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    let mut row = g(PIXELS);
    for _ in 0..g(HEIGHT) {
        let dwords = g(ROW_DWORDS);
        unsafe { std::ptr::write_bytes(row as *mut u32, 0, dwords as usize) };
        row = row.wrapping_add(4 * dwords).wrapping_add(g(GAP));
    }
    r.eax = 0;
    r.edx = 0;
    r.ecx = 0;
    r.edi = row;
    part(2, &mut r)?;
    Ok(*entry)
}

/// In the game: the platform's lock and unlock through the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    let at = if place == 1 {
        crate::gfx::take_screenshot::LOCK
    } else {
        crate::gfx::take_screenshot::UNLOCK
    };
    crate::calls::call(rva(at), r);
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        clear_back_surface(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; cmp g_frontSurface, 0
    let copy = match crate::race::viewer::copy(
        0x40579A,
        0x37,
        &[0x60, 0x83, 0x3D, 0xB4, 0x32, 0x4F, 0x00, 0x00],
        &[0xA, 0x30],
        &[
            (crate::gfx::take_screenshot::LOCK, 1),
            (crate::gfx::take_screenshot::UNLOCK, 2),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let vars = [
        dword(FRONT),
        dword(PIXELS),
        dword(HEIGHT),
        dword(ROW_DWORDS),
        dword(GAP),
    ];
    let saved = Saved::new(&vars);
    let mut buffer = vec![0xAAu8; 0x400];
    let at = buffer.as_mut_ptr() as u32;
    for (front, fails, height, dwords, gap) in [
        (0u32, false, 4u32, 8u32, 0u32),
        (1, true, 4, 8, 0),
        (1, false, 4, 8, 0x10),
        (1, false, 6, 3, 4),
    ] {
        run_both(
            &mut t,
            &format!("front {front}, fails {fails}, {height} rows of {dwords} dwords, gap {gap}"),
            copy.addr(),
            Regs::SENTINEL,
            &[(at, 0x400)],
            0,
            &|| {
                unsafe { std::ptr::write_bytes(at as *mut u8, 0xAA, 0x400) };
                p(FRONT, front);
                p(PIXELS, at + 8);
                p(HEIGHT, height);
                p(ROW_DWORDS, dwords);
                p(GAP, gap);
                answer(Box::new(move |place, r| {
                    if place == 1 {
                        r.set_carry(fails);
                    }
                }));
            },
            &|r| clear_back_surface(r, &mut test_part),
        );
    }
    crate::race::viewer::quiet();
    drop(saved);
    drop(buffer);
    t.finish()
}
