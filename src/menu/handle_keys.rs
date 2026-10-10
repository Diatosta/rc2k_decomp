//! `menu::handle_keys` (0x465FCC): takes one key and does what it means on a menu.
//!
//! In:  nothing; but bits 8 .. 15 of eax are read on Enter (see below)
//! Out: eax = 1 nothing to do for the caller, 2 Escape, 3 Enter; edx = the frame clock after
//!      Enter and after any key not named here first; edi = `dword_731BBB` after one of the
//!      four moves; everything else preserved
//! Nothing happens (eax = 1) while an item is being edited (`dword_75AA9C` = 1) or
//! `dword_7551F4` is 1. Otherwise `dword_7556A4` = 0x6F, `dword_755561` = 0, and the key:
//!   none        eax = 1
//!   any         first `menu::handle_yes_no`
//!   0x38        nothing
//!   1 (Escape)  `dword_755561`, `dword_755A3C` and `dword_758806` = 1; eax = 2
//!   0x1C (Enter) `dword_7556A4` = 0x14D, the last-key time `dword_7556C4` = the clock,
//!               `dword_7554D4` = 1 if ah is 4 and else 0, the sound 2; eax = 3. `take_key`
//!               only changes al, so ah is what the caller had in eax: reproduced as it is.
//!   others      the last-key time = the clock, then
//!     0x66, 0x6B, 0x69, 0x68  the four `move_highlight_*`
//!     0x52      `sub_407897` (a screenshot, by what it calls)
//!     0x20      flips `dword_755575`
//!     0x14      adds 60 to the service time budget `dword_75593C`, if bit 14 of
//!               `dword_75870C` is set
//! The code after the Escape branch that could return 0 is never reached.
//! 89 callers: every screen loop. Shadow mode: the original runs for real, then the Rust
//! version on the same key without the sound, the callbacks and the screenshot; compared
//! unless a callback or the screenshot was left out. Needs the `input::take_key` hook on.
//! Self-test: a redirected copy whose key, sound, screenshot and moves are stand-ins.
//! Refuses, on a yes/no screen, a missing keyboard layout (before the key is taken).

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EDI, EDX, RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm,
    rva,
};
use crate::input::key_to_char::has_layout;
use crate::input::take_key::{replaying_key, take_key};
use crate::menu::handle_yes_no::{ASKING_RVA, LEAVING_RVA, handle_yes_no, handle_yes_no_regions};
use crate::menu::navigation::{
    CLICK_RVA, Direction, Noting, Outside, SCREENSHOT_RVA, TEST_KEY, clear_notes, click_stand_in,
    dispatch_outside, down_stand_in, left_stand_in, move_regions, notes, right_stand_in,
    screenshot_stand_in, take_key_stand_in, up_stand_in,
};
use crate::menu::update_blink::LAST_KEY_TIME_RVA;
use crate::menu::update_frame_clock::CLOCK_RVA;
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, restore};
use crate::service::state::{EDIT_MODE_RVA, TIME_BUDGET_RVA};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::handle_keys", rva(0x465FCC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const KEYS_OFF_RVA: usize = rva(0x7551F4); // dword_7551F4: 1 = keys are not looked at
const SOUND_RVA: usize = rva(0x7556A4); // dword_7556A4: 0x6F, or 0x14D after Enter
const BACK_RVA: usize = rva(0x755561); // dword_755561
const ESCAPED_RVA: usize = rva(0x755A3C); // dword_755A3C
const ENTER_WITH_4_RVA: usize = rva(0x7554D4); // dword_7554D4
const TOGGLE_RVA: usize = rva(0x755575); // dword_755575: flipped by key 0x20
const SWITCHES_RVA: usize = rva(0x75870C); // dword_75870C: bit 14 lets key 0x14 add time
const MENU_RVA: usize = rva(0x731BBB); // dword_731BBB

const NO_SOUND: u32 = 0x6F;
const ENTER_SOUND: u32 = 0x14D;

/// What the function and the ones under it may write.
fn regions() -> Vec<Region> {
    [
        &[
            at(SOUND_RVA),
            at(BACK_RVA),
            at(ESCAPED_RVA),
            at(ENTER_WITH_4_RVA),
            at(TOGGLE_RVA),
            at(TIME_BUDGET_RVA),
            at(LAST_KEY_TIME_RVA),
        ][..],
        &handle_yes_no_regions(),
        &move_regions(),
    ]
    .concat()
}

// ---- Rust version ----

/// `entry` are the registers at entry. Returns what is left in the registers.
pub unsafe fn handle_keys(entry: &Regs, out: &mut dyn Outside) -> Result<Left, Refused> {
    let nothing = Left::nothing().with(EAX, 1);
    unsafe {
        if game_read::<u32>(EDIT_MODE_RVA) == 1 || game_read::<u32>(KEYS_OFF_RVA) == 1 {
            return Ok(nothing);
        }
        if game_read::<u32>(ASKING_RVA) == 1 && !has_layout() {
            return Err(BadInput("no keyboard layout").into());
        }
        game_write(SOUND_RVA, NO_SOUND);
        game_write(BACK_RVA, 0u32);
        let key = take_key();
        if key == 0 {
            return Ok(nothing);
        }
        let eax = entry.eax & !0xFF | key as u32;
        handle_yes_no(eax)?;
        if key == 0x38 {
            return Ok(nothing);
        }
        if key == 1 {
            game_write(BACK_RVA, 1u32);
            game_write(ESCAPED_RVA, 1u32);
            game_write(LEAVING_RVA, 1u32);
            return Ok(Left::nothing().with(EAX, 2));
        }
        let clock: u32 = game_read(CLOCK_RVA);
        let regs = Regs {
            eax,
            edx: clock,
            ..*entry
        };
        if key == 0x1C {
            game_write(SOUND_RVA, ENTER_SOUND);
            game_write(LAST_KEY_TIME_RVA, clock);
            game_write(ENTER_WITH_4_RVA, (eax >> 8 & 0xFF == 4) as u32);
            out.click(&Regs { eax: 2, ..regs });
            return Ok(Left::nothing().with(EAX, 3).with(EDX, clock));
        }
        game_write(LAST_KEY_TIME_RVA, clock);
        let nothing = nothing.with(EDX, clock);
        let direction = match key {
            0x66 => Direction::Up,
            0x6B => Direction::Down,
            0x69 => Direction::Right,
            0x68 => Direction::Left,
            0x52 => {
                out.screenshot(&regs);
                return Ok(nothing);
            }
            0x20 => {
                game_write(TOGGLE_RVA, game_read::<u32>(TOGGLE_RVA) ^ 1);
                return Ok(nothing);
            }
            0x14 if game_read::<u32>(SWITCHES_RVA) >> 14 & 1 != 0 => {
                game_write(
                    TIME_BUDGET_RVA,
                    game_read::<u32>(TIME_BUDGET_RVA).wrapping_add(60),
                );
                return Ok(nothing);
            }
            _ => return Ok(nothing),
        };
        let menu: u32 = game_read(MENU_RVA);
        out.move_highlight(direction, &Regs { edi: menu, ..regs })?;
        Ok(nothing.with(EDI, menu))
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    // The console's `fatal` test fires at the next key poll, and with this hook's Rust
    // version running the menus no longer pass through the `take_key` hook.
    crate::fatal::raise::raise_test_if_armed(r);
    dispatch_outside(
        &HOOK,
        r,
        call_original,
        &regions(),
        &|r: &Regs, out: &mut dyn Outside| unsafe { handle_keys(r, out) },
        true,
    );
}

// ---- self-test: a copy whose key, sound, screenshot and moves are stand-ins ----

const FUNCTION_LEN: usize = 0x173;
const ENTRY_BYTES: [u8; 8] = [0x83, 0x3D, 0x9C, 0xAA, 0x75, 0x00, 0x01, 0x0F];
const CALLS: [usize; 8] = [0x2E, 0x39, 0x9B, 0xBB, 0xCF, 0xE3, 0xF4, 0xFF];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let stand_in = |function: unsafe extern "C" fn()| Some(function as *const () as usize);
    let redirect = |target: usize| match target {
        t if t == rva(0x407F95) => stand_in(take_key_stand_in),
        t if t == CLICK_RVA => stand_in(click_stand_in),
        t if t == SCREENSHOT_RVA => stand_in(screenshot_stand_in),
        t if t == rva(0x4746C4) => stand_in(up_stand_in),
        t if t == rva(0x474715) => stand_in(down_stand_in),
        t if t == rva(0x474766) => stand_in(right_stand_in),
        t if t == rva(0x4747EE) => stand_in(left_stand_in),
        _ => None,
    };
    let copy = match unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x465FCC)),
            FUNCTION_LEN,
            &ENTRY_BYTES,
            &CALLS,
            &redirect,
        )
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };

    let regions = regions();
    let inputs = [
        at(EDIT_MODE_RVA),
        at(KEYS_OFF_RVA),
        at(SWITCHES_RVA),
        at(ASKING_RVA),
        at(MENU_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let switches: u32 = unsafe { game_read(SWITCHES_RVA) };
    // A yes/no screen needs a keyboard layout and the strings.
    let askings: &[u32] = if crate::input::yes_no_key::yes_no_unavailable().is_none() {
        &[0, 1]
    } else {
        &[0]
    };

    // (edit mode, keys off)
    for (editing, keys_off) in [(0u32, 0u32), (1, 0), (0, 1)] {
        for key in [
            0u32, 1, 0x1C, 0x38, 0x66, 0x6B, 0x69, 0x68, 0x52, 0x20, 0x14, 0x15, 0x31,
        ] {
            for high in [0x5555_0400u32, 0x1234_0000] {
                for add_time in [false, true] {
                    for &asking in askings {
                        let prepare = || unsafe {
                            restore(&regions, &saved.0);
                            game_write(EDIT_MODE_RVA, editing);
                            game_write(KEYS_OFF_RVA, keys_off);
                            game_write(
                                SWITCHES_RVA,
                                switches & !(1 << 14) | (add_time as u32) << 14,
                            );
                            game_write(ASKING_RVA, asking);
                            game_write(MENU_RVA, 0xABCD_0000u32);
                            for at in [ESCAPED_RVA, ENTER_WITH_4_RVA, LEAVING_RVA] {
                                game_write(at, 0x5EEDu32);
                            }
                            TEST_KEY.store(key, Ordering::SeqCst);
                            clear_notes();
                        };
                        let before = Regs {
                            eax: high,
                            ..Regs::SENTINEL
                        };
                        prepare();
                        let mut ro = before;
                        unsafe { call_game(&mut ro, copy.addr()) };
                        let theirs = (capture(&regions), notes());
                        prepare();
                        let outcome = replaying_key(key as u8, || unsafe {
                            handle_keys(&before, &mut Noting)
                        });
                        let mine = (capture(&regions), notes());
                        let regs_ok = outcome
                            .as_ref()
                            .is_ok_and(|left| left.matches(&before, &ro));
                        t.check(regs_ok && theirs == mine, || {
                            format!(
                                "key={key:#x} eax={high:#x} editing={editing} keys_off={keys_off} add_time={add_time} asking={asking}: regs_ok={regs_ok} original eax={:#x} edx={:#x} edi={:#x} {theirs:x?}; rust {outcome:?} {mine:x?}",
                                ro.eax, ro.edx, ro.edi
                            )
                        });
                    }
                }
            }
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
