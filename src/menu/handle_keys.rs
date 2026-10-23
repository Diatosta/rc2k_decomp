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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EDI, EDX, Regs, game_read, game_write, rva};
use crate::input::key_to_char::has_layout;
use crate::input::take_key::take_key;
use crate::menu::handle_yes_no::{ASKING_RVA, LEAVING_RVA, handle_yes_no};
use crate::menu::navigation::{Direction, Outside};
use crate::menu::update_blink::LAST_KEY_TIME_RVA;
use crate::menu::update_frame_clock::CLOCK_RVA;
use crate::screens::harness::Left;
use crate::service::state::{EDIT_MODE_RVA, TIME_BUDGET_RVA};

// ---- game data ----

const KEYS_OFF_RVA: usize = rva(0x7551F4); // dword_7551F4: 1 = keys are not looked at
const SOUND_RVA: usize = rva(0x7556A4); // dword_7556A4: 0x6F, or 0x14D after Enter
const BACK_RVA: usize = rva(0x755561); // dword_755561
const ESCAPED_RVA: usize = rva(0x755A3C); // dword_755A3C
const ENTER_WITH_4_RVA: usize = rva(0x7554D4); // dword_7554D4
const TOGGLE_RVA: usize = rva(0x755575); // dword_755575: flipped by key 0x20
const SWITCHES_RVA: usize = rva(0x75870C); // dword_75870C: bit 14 lets key 0x14 add time
const MENU_RVA: usize = rva(0x731BBB); // dword_731BBB

// ---- Rust version ----

/// `entry` are the registers at entry. Returns what is left in the registers.
pub unsafe fn handle_keys(entry: &Regs, out: &mut dyn Outside) -> Result<Left, Refused> {
    // Converted: `rc2k_core::menu::keys` (`keys_off`, `action`, `entered_with_4`).
    use rc2k_core::menu::keys::{self, Action, Move};
    let nothing = Left::nothing().with(EAX, keys::NOTHING);
    unsafe {
        if keys::keys_off(game_read(EDIT_MODE_RVA), || game_read(KEYS_OFF_RVA)) {
            return Ok(nothing);
        }
        if game_read::<u32>(ASKING_RVA) == 1 && !has_layout() {
            return Err(BadInput("no keyboard layout").into());
        }
        game_write(SOUND_RVA, keys::NO_SOUND);
        game_write(BACK_RVA, 0u32);
        let key = take_key();
        if key == 0 {
            return Ok(nothing);
        }
        let eax = entry.eax & !0xFF | key as u32;
        handle_yes_no(eax)?;
        let action = keys::action(key, || game_read(SWITCHES_RVA));
        match action {
            Action::Ignored => return Ok(nothing),
            Action::Escape => {
                game_write(BACK_RVA, 1u32);
                game_write(ESCAPED_RVA, 1u32);
                game_write(LEAVING_RVA, 1u32);
                return Ok(Left::nothing().with(EAX, keys::ESCAPED));
            }
            _ => {}
        }
        let clock: u32 = game_read(CLOCK_RVA);
        let regs = Regs {
            eax,
            edx: clock,
            ..*entry
        };
        if action == Action::Enter {
            game_write(SOUND_RVA, keys::ENTER_SOUND);
            game_write(LAST_KEY_TIME_RVA, clock);
            game_write(ENTER_WITH_4_RVA, keys::entered_with_4(eax));
            out.click(&Regs {
                eax: keys::ENTER_CLICK,
                ..regs
            });
            return Ok(Left::nothing().with(EAX, keys::ENTERED).with(EDX, clock));
        }
        game_write(LAST_KEY_TIME_RVA, clock);
        let nothing = nothing.with(EDX, clock);
        let direction = match action {
            Action::Move(Move::Up) => Direction::Up,
            Action::Move(Move::Down) => Direction::Down,
            Action::Move(Move::Right) => Direction::Right,
            Action::Move(Move::Left) => Direction::Left,
            Action::Screenshot => {
                out.screenshot(&regs);
                return Ok(nothing);
            }
            Action::Toggle => {
                game_write(TOGGLE_RVA, game_read::<u32>(TOGGLE_RVA) ^ 1);
                return Ok(nothing);
            }
            Action::AddTime => {
                game_write(
                    TIME_BUDGET_RVA,
                    game_read::<u32>(TIME_BUDGET_RVA).wrapping_add(keys::ADDED_TIME),
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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::menu::navigation::outside_entry(r, &|r: &Regs, out: &mut dyn Outside| unsafe {
        handle_keys(r, out)
    })
}
