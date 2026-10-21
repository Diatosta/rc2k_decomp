//! `gfx::keep_race_screen_mode` (0x404805): mode eax kept as the race's
//! (`g_raceScreenMode`), and its pixel kind (the second byte) with 0x320002 (800 x 600 by
//! its size bits, the menus' other bits) tried (`gfx::try_screen_mode`): if that works it
//! becomes the menus' mode (`g_menuScreenMode`).
//!
//! In:  eax = the mode
//! Out: every register kept (`pusha` .. `popa`); the flags as `try_screen_mode` leaves them
//!
//! One caller, `gfx::screen_mode_choices`. Its call through `part` (0; in both programs
//! `try_screen_mode`'s Rust version). Shadow mode runs the original alone (the trial sets
//! the screen). Self-test: a copy whose call goes to a stand-in (working or not), against
//! the Rust version.

use crate::errors::Refused;
use crate::framework::{CARRY_FLAG, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{Part, Saved, answer, dispatch_parts, dword, put, run_both, test_part};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "gfx::keep_race_screen_mode",
    rva(0x404805),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const RACE_MODE: usize = 0x4F4948; // g_raceScreenMode
pub const MENU_MODE: usize = 0x4F4944; // g_menuScreenMode

/// The Rust version: the registers and flags it leaves.
pub fn keep_race_screen_mode(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    put(RACE_MODE, entry.eax);
    // Converted: `rc2k_core::gfx::screen_modes::menu_mode_for`.
    let mut r = Regs {
        eax: rc2k_core::gfx::screen_modes::menu_mode_for(entry.eax),
        ..*entry
    };
    part(0, &mut r)?;
    if !r.carry() {
        put(MENU_MODE, r.eax);
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// In both programs: `try_screen_mode`'s Rust version (its calls through their hooks).
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    *r = crate::gfx::try_screen_mode::try_screen_mode(
        r,
        &mut crate::gfx::try_screen_mode::game_part,
    )?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        keep_race_screen_mode(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov g_raceScreenMode, eax
    let copy = match crate::race::viewer::copy(
        0x404805,
        0x1E,
        &[0x60, 0xA3, 0x48, 0x49, 0x4F, 0x00],
        &[0x10],
        &[(0x404AED, 0)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [dword(RACE_MODE), dword(MENU_MODE)];
    let saved = Saved::new(&regions);
    for mode in [0x34_1880u32, 0x32_1802, 0xFFFF_FFFF] {
        for works in [false, true] {
            run_both(
                &mut t,
                &format!("mode {mode:#x}, the trial works {works}"),
                copy.addr(),
                Regs {
                    eax: mode,
                    ..Regs::SENTINEL
                },
                &regions,
                CARRY_FLAG,
                &|| {
                    put(RACE_MODE, 0x5EED);
                    put(MENU_MODE, 0x5EED);
                    answer(Box::new(move |_, r| r.set_carry(!works)));
                },
                &|r| keep_race_screen_mode(r, &mut test_part),
            );
        }
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
