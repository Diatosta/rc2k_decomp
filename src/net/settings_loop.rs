//! `net::settings_loop` (0x4168CF): the network settings screen (its menu's action 0x4AAADB):
//! `dword_52A908` 0, the background (`net::show_background`), the saved connection checked
//! (`net::settings_done`); the list's clocks from `dword_518998`, no editing, item 0. Each
//! frame (`net::settings_frame` through `screens::run_frame`): the frame clock noted, a key
//! (`menu::list_take_key`); while a setting is edited the key goes to it
//! (`net::setting_key`); else up (0x66) and down (0x6B) move the highlight within 0 .. 4
//! (the move clock then noted), Enter (0x1C) on item 4 leaves, on item 0 chooses the
//! connection (`net::settings_link`, the highlight back to 0), on the others starts the
//! editing (`net::begin_setting_edit`); Escape (1) leaves; 'R' (0x52) with screenshots on
//! takes one (`gfx::take_screenshot`); any other key may compile the lexicon again
//! (`strings::maybe_compile_lexicon`). It ends too when the game is closing; then
//! `g_listChoice` -1.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Its calls through `part` (the frames through the game). The whole screen runs inside it:
//! ungated. Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins (the keys scripted, Escape at the end), against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    CHOICE, CLOCK, EDITED, FRAME_CLOCK, KEY_CLOCK, LIST_TAKE_KEY, MAYBE_LEXICON, MODE, MOVE_CLOCK,
    Part, QUITTING, RUN_FRAME, SCREENSHOT, SCREENSHOTS_ON, g, game, lobby_entry, p,
};

const SHOW_BACKGROUND: u32 = 0x4173E7; // net::show_background
const SETTINGS_DONE: u32 = 0x416F0B; // net::settings_done
const SETTINGS_FRAME: u32 = 0x4169E2; // net::settings_frame
const SETTING_KEY: u32 = 0x416C76; // net::setting_key
const SETTINGS_LINK: u32 = 0x416EEC; // net::settings_link
const BEGIN_SETTING_EDIT: u32 = 0x416C26; // net::begin_setting_edit

/// The Rust version: the registers it leaves (the entry's).
pub fn settings_loop(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(MODE, 0);
    part(SHOW_BACKGROUND, &mut r)?;
    part(SETTINGS_DONE, &mut r)?;
    r.eax = g(CLOCK);
    p(MOVE_CLOCK, r.eax);
    p(KEY_CLOCK, r.eax);
    p(EDITED, u32::MAX);
    p(CHOICE, 0);
    loop {
        r.eax = game(SETTINGS_FRAME);
        part(RUN_FRAME, &mut r)?;
        r.eax = g(CLOCK);
        p(FRAME_CLOCK, r.eax);
        part(LIST_TAKE_KEY, &mut r)?;
        let mut moved = false;
        if g(EDITED) != u32::MAX {
            part(SETTING_KEY, &mut r)?;
        } else {
            match r.eax as u8 {
                0x66 => {
                    let c = g(CHOICE).wrapping_sub(1);
                    p(CHOICE, if (c as i32) < 0 { 0 } else { c });
                    moved = true;
                }
                0x6B => {
                    let c = g(CHOICE).wrapping_add(1);
                    p(CHOICE, if (c as i32) > 4 { 4 } else { c });
                    moved = true;
                }
                0x1C => {
                    if g(CHOICE) == 4 {
                        break;
                    }
                    if g(CHOICE) == 0 {
                        part(SETTINGS_LINK, &mut r)?;
                        p(CHOICE, 0);
                    } else {
                        part(BEGIN_SETTING_EDIT, &mut r)?;
                    }
                    moved = true;
                }
                1 => break,
                key => {
                    if g(SCREENSHOTS_ON) != 0 && key == 0x52 {
                        part(SCREENSHOT, &mut r)?;
                    }
                    part(MAYBE_LEXICON, &mut r)?;
                }
            }
        }
        if moved {
            r.ebx = g(FRAME_CLOCK);
            p(MOVE_CLOCK, r.ebx);
        }
        if g(QUITTING) == 1 {
            break;
        }
    }
    p(CHOICE, u32::MAX);
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, settings_loop)
}
