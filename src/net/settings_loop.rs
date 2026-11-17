//! The network settings screen: up and down move the highlight over seven items, Enter leaves
//! (item 5), chooses the connection (item 0) or starts editing the setting; Escape leaves. A
//! setting being edited gets the keys. Also ends when the game is closing.

use crate::errors::Refused;
use crate::net::lobby::{list_key, other_key, run_frame};
/// The last item, the one that leaves (the password is the item before it).
const LAST: u32 = 6;

pub fn settings_loop(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (net, system) = (o.net, o.system);
    let list_state = &o.menus.list;
    net.lobby().mode.set(0);
    crate::net::show_background::show_background(o)?;
    crate::net::settings_done::settings_done(o, &crate::race::viewer_screen_store::NET_TEXTS)?;
    let now = system.session.key_clock.ticks.get();
    list_state.move_clock.set(now);
    list_state.key_clock.set(now);
    net.settings().edit.edited.set(u32::MAX);
    list_state.choice.set(0);
    loop {
        run_frame(o, || crate::net::settings_frame::settings_frame(o))?;
        list_state
            .frame_clock
            .set(system.session.key_clock.ticks.get());
        let key = list_key(list_state, o.input)?;
        let mut moved = false;
        if net.settings().edit.edited.get() != u32::MAX {
            crate::net::setting_key::setting_key(o.input, o.strings, list_state, net, key, o.gfx)?;
        } else {
            match key {
                0x66 => {
                    let c = list_state.choice.get().wrapping_sub(1);
                    list_state.choice.set(if (c as i32) < 0 { 0 } else { c });
                    moved = true;
                }
                0x6B => {
                    let c = list_state.choice.get().wrapping_add(1);
                    list_state.choice.set(if c > LAST { LAST } else { c });
                    moved = true;
                }
                0x1C => {
                    if list_state.choice.get() == LAST {
                        break;
                    }
                    if list_state.choice.get() == 0 {
                        crate::net::settings_link::settings_link(
                            o,
                            &crate::race::viewer_screen_store::NET_TEXTS,
                        )?;
                        list_state.choice.set(0);
                    } else {
                        crate::net::begin_setting_edit::begin_setting_edit(net, list_state)?;
                    }
                    moved = true;
                }
                1 => break,
                key => other_key(o, key)?,
            }
        }
        if moved {
            list_state.move_clock.set(list_state.frame_clock.get());
        }
        if system.session.flags.quitting.is_set() {
            break;
        }
    }
    list_state.choice.set(u32::MAX);
    Ok(())
}
