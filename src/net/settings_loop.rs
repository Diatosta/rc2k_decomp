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

use crate::errors::Refused;
use crate::net::lobby::{list_key, other_key, run_frame};

/// The Rust version.
pub fn settings_loop() -> Result<(), Refused> {
    crate::net::owner::net().vars.mode.set(0);
    crate::net::show_background::show_background()?;
    crate::net::settings_done::settings_done()?;
    let now = crate::system::owner::system()
        .session
        .clocks
        .key_ticks
        .get();
    crate::install::owner::install().pages.move_clock.set(now);
    crate::install::owner::install().pages.key_clock.set(now);
    crate::net::owner::net().vars.edited.set(u32::MAX);
    crate::install::owner::install().pages.choice.set(0);
    loop {
        run_frame(crate::net::settings_frame::settings_frame)?;
        crate::install::owner::install().pages.frame_clock.set(
            crate::system::owner::system()
                .session
                .clocks
                .key_ticks
                .get(),
        );
        let key = list_key()?;
        let mut moved = false;
        if crate::net::owner::net().vars.edited.get() != u32::MAX {
            crate::net::setting_key::setting_key(key)?;
        } else {
            match key {
                0x66 => {
                    let c = crate::install::owner::install()
                        .pages
                        .choice
                        .get()
                        .wrapping_sub(1);
                    crate::install::owner::install()
                        .pages
                        .choice
                        .set(if (c as i32) < 0 { 0 } else { c });
                    moved = true;
                }
                0x6B => {
                    let c = crate::install::owner::install()
                        .pages
                        .choice
                        .get()
                        .wrapping_add(1);
                    crate::install::owner::install()
                        .pages
                        .choice
                        .set(if (c as i32) > 4 { 4 } else { c });
                    moved = true;
                }
                0x1C => {
                    if crate::install::owner::install().pages.choice.get() == 4 {
                        break;
                    }
                    if crate::install::owner::install().pages.choice.get() == 0 {
                        crate::net::settings_link::settings_link()?;
                        crate::install::owner::install().pages.choice.set(0);
                    } else {
                        crate::net::begin_setting_edit::begin_setting_edit()?;
                    }
                    moved = true;
                }
                1 => break,
                key => other_key(key)?,
            }
        }
        if moved {
            crate::install::owner::install()
                .pages
                .move_clock
                .set(crate::install::owner::install().pages.frame_clock.get());
        }
        if crate::system::owner::system().session.flags.quitting.get() == 1 {
            break;
        }
    }
    crate::install::owner::install().pages.choice.set(u32::MAX);
    Ok(())
}
