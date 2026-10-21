//! `gfx::take_screenshot` (0x407897): with screenshots on (`g_screenshotsOn`), the screen
//! saved as `000.pcx` and on: the surface locked unless it is (`gfx::lock_surface`; a
//! failed lock ends it; `dword_4F52B8` 1 while it is this function's lock), written
//! (`gfx::write_screen_pcx`), unlocked again if it was locked here
//! (`gfx::unlock_surface`), `dword_4F52B8` 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::replay_viewer` (key 0x52), `gfx::screenshot_if_key`,
//! `menu::handle_keys`, `sub_4146E4`, `sub_4168CF`, `sub_4174C5`, `sub_417726`,
//! `sub_417A0D`, 0x471CA6. Its calls through `part`: 1 the lock (carry), 2 the writer, 3
//! the unlock; in the game the platform's two through the game (their hooks in the
//! standalone) and the writer's Rust version. Shadow mode runs the original alone (it
//! writes a file). Self-test: a copy whose calls go to stand-ins, against the Rust
//! version: off, on with the surface locked or not, the lock failing.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::race::viewer::{
    Part, Saved, answer, dispatch_parts, dword, get, put, run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::take_screenshot", rva(0x407897), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SCREENSHOTS_ON: usize = 0x4F52B4; // g_screenshotsOn
const LOCKED: usize = 0x4F3870; // g_surfaceLocked
const LOCKED_HERE: usize = 0x4F52B8; // dword_4F52B8
pub const LOCK: usize = 0x4058AD; // gfx::lock_surface
pub const UNLOCK: usize = 0x405949; // gfx::unlock_surface

/// The parts in the game's memory, the registers carried from call to call.
struct Shot<'a> {
    r: Regs,
    part: Part<'a>,
}

impl rc2k_core::gfx::screenshot::Shooting for Shot<'_> {
    type Error = Refused;
    fn screenshots_on(&mut self) -> u32 {
        get(SCREENSHOTS_ON)
    }
    fn surface_locked(&mut self) -> u32 {
        get(LOCKED)
    }
    fn locked_here(&mut self) -> u32 {
        get(LOCKED_HERE)
    }
    fn set_locked_here(&mut self, value: u32) {
        put(LOCKED_HERE, value);
    }
    fn lock(&mut self) -> Result<bool, Refused> {
        (self.part)(1, &mut self.r)?;
        Ok(self.r.carry())
    }
    fn write(&mut self) -> Result<(), Refused> {
        (self.part)(2, &mut self.r)
    }
    fn unlock(&mut self) -> Result<(), Refused> {
        (self.part)(3, &mut self.r)
    }
}

/// The Rust version: the registers it leaves (the entry's).
pub fn take_screenshot(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::gfx::screenshot::take_screenshot`.
    rc2k_core::gfx::screenshot::take_screenshot(&mut Shot { r: *entry, part })?;
    Ok(*entry)
}

/// In the game: the platform's lock and unlock through the game, the writer's Rust
/// version.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1 => unsafe { call_game(r, game_addr(rva(LOCK))) },
        2 => unsafe { crate::gfx::write_screen_pcx::write_screen_pcx_in_game(r) }?,
        _ => unsafe { call_game(r, game_addr(rva(UNLOCK))) },
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        take_screenshot(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; cmp g_screenshotsOn, 0
    let copy = match crate::race::viewer::copy(
        0x407897,
        0x43,
        &[0x60, 0x83, 0x3D, 0xB4, 0x52, 0x4F, 0x00, 0x00],
        &[0x13, 0x24, 0x32],
        &[(LOCK, 1), (0x411E0C, 2), (UNLOCK, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [dword(SCREENSHOTS_ON), dword(LOCKED), dword(LOCKED_HERE)];
    let saved = Saved::new(&regions);
    for (on, locked, fails) in [
        (0u32, 0u32, false),
        (1, 0, false),
        (1, 1, false),
        (1, 0, true),
    ] {
        run_both(
            &mut t,
            &format!("on {on}, locked {locked}, the lock failing {fails}"),
            copy.addr(),
            Regs::SENTINEL,
            &regions,
            0,
            &|| {
                put(SCREENSHOTS_ON, on);
                put(LOCKED, locked);
                put(LOCKED_HERE, 7);
                answer(Box::new(move |place, r| {
                    if place == 1 {
                        r.set_carry(fails);
                    }
                }));
            },
            &|r| take_screenshot(r, &mut test_part),
        );
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
