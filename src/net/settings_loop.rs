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
//! Out: nothing (the original keeps every register: `pusha` .. `popa`)
//! Its calls through `part` (the frames through the game). The whole screen runs inside it:
//! ungated. Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins (the keys scripted, Escape at the end), against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::net::lobby::{
    CHOICE, EDITED, FRAME_CLOCK, KEY_CLOCK, MODE, MOVE_CLOCK, g, list_key, other_key, p, run_frame,
};
use crate::session::CLOCKS;
use crate::session::FLAGS;

/// The Rust version.
pub fn settings_loop() -> Result<(), Refused> {
    p(MODE, 0);
    crate::net::show_background::show_background()?;
    crate::net::settings_done::settings_done()?;
    let now = CLOCKS.key_ticks.get();
    p(MOVE_CLOCK, now);
    p(KEY_CLOCK, now);
    p(EDITED, u32::MAX);
    p(CHOICE, 0);
    loop {
        run_frame(crate::net::settings_frame::settings_frame)?;
        p(FRAME_CLOCK, CLOCKS.key_ticks.get());
        let key = list_key()?;
        let mut moved = false;
        if g(EDITED) != u32::MAX {
            crate::net::setting_key::setting_key(key)?;
        } else {
            match key {
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
                        crate::net::settings_link::settings_link()?;
                        p(CHOICE, 0);
                    } else {
                        crate::net::begin_setting_edit::begin_setting_edit();
                    }
                    moved = true;
                }
                1 => break,
                key => other_key(key)?,
            }
        }
        if moved {
            p(MOVE_CLOCK, g(FRAME_CLOCK));
        }
        if FLAGS.quitting.get() == 1 {
            break;
        }
    }
    p(CHOICE, u32::MAX);
    Ok(())
}
