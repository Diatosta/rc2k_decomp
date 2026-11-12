//! `net::settings_loop` (0x4168CF): the network settings screen (its menu's action 0x4AAADB):
//! `mode` 0, the background (`net::show_background`), the saved connection checked
//! (`net::settings_done`); the list's clocks from `clocks.key_ticks`, no editing, item 0. Each
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

use crate::errors::Refused;
use crate::net::lobby::{list_key, other_key, run_frame};
pub fn settings_loop(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (install, net, system) = (o.install, o.net, o.system);
    let vars = &net.vars;
    vars.mode.set(0);
    crate::net::show_background::show_background(o.gfx, o.system, &net.vars)?;
    crate::net::settings_done::settings_done(net, system, o.race.viewer_panel())?;
    let now = system.session.clocks.key_ticks.get();
    install.pages.move_clock.set(now);
    install.pages.key_clock.set(now);
    net.edit().edited.set(u32::MAX);
    install.pages.choice.set(0);
    loop {
        run_frame(|| crate::net::settings_frame::settings_frame(o))?;
        install
            .pages
            .frame_clock
            .set(system.session.clocks.key_ticks.get());
        let key = list_key()?;
        let mut moved = false;
        if net.edit().edited.get() != u32::MAX {
            crate::net::setting_key::setting_key(install, net, key)?;
        } else {
            match key {
                0x66 => {
                    let c = install.pages.choice.get().wrapping_sub(1);
                    install.pages.choice.set(if (c as i32) < 0 { 0 } else { c });
                    moved = true;
                }
                0x6B => {
                    let c = install.pages.choice.get().wrapping_add(1);
                    install.pages.choice.set(if (c as i32) > 4 { 4 } else { c });
                    moved = true;
                }
                0x1C => {
                    if install.pages.choice.get() == 4 {
                        break;
                    }
                    if install.pages.choice.get() == 0 {
                        crate::net::settings_link::settings_link(o, o.race.viewer_panel())?;
                        install.pages.choice.set(0);
                    } else {
                        crate::net::begin_setting_edit::begin_setting_edit(net, install)?;
                    }
                    moved = true;
                }
                1 => break,
                key => other_key(o, key)?,
            }
        }
        if moved {
            install
                .pages
                .move_clock
                .set(install.pages.frame_clock.get());
        }
        if system.session.flags.quitting.is_set() {
            break;
        }
    }
    install.pages.choice.set(u32::MAX);
    Ok(())
}
