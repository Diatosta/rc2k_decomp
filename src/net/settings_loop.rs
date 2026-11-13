//! The network settings screen: up and down move the highlight over five items, Enter leaves
//! (item 4), chooses the connection (item 0) or starts editing the setting; Escape leaves. A
//! setting being edited gets the keys. Also ends when the game is closing.

use crate::errors::Refused;
use crate::net::lobby::{list_key, other_key, run_frame};
pub fn settings_loop(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (install, net, system) = (o.install, o.net, o.system);
    net.lobby().mode.set(0);
    crate::net::show_background::show_background(o)?;
    crate::net::settings_done::settings_done(o, &crate::race::viewer_screen_store::NET_TEXTS)?;
    let now = system.session.key_clock.ticks.get();
    install.pages.move_clock.set(now);
    install.pages.key_clock.set(now);
    net.edit().edited.set(u32::MAX);
    install.pages.choice.set(0);
    loop {
        run_frame(o, || crate::net::settings_frame::settings_frame(o))?;
        install
            .pages
            .frame_clock
            .set(system.session.key_clock.ticks.get());
        let key = list_key(install, o.input)?;
        let mut moved = false;
        if net.edit().edited.get() != u32::MAX {
            crate::net::setting_key::setting_key(o.input, o.strings, install, net, key, o.gfx)?;
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
                        crate::net::settings_link::settings_link(
                            o,
                            &crate::race::viewer_screen_store::NET_TEXTS,
                        )?;
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
