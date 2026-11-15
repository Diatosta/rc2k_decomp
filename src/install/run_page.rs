//! One of the installer's pages (a list menu) run until it is left: the list's action is told the
//! start (state 0); a list with 0x40 or 0x80 is only that action. Else the items are marked, the
//! highlight moved past items that cannot be chosen, the backdrop shown; then each frame: the
//! pieces laid out, measured, placed, the page drawn, a key taken. A key goes to the item
//! (`install::page_key`); if it did nothing, to the list's key action; if that did nothing
//! either, it moves the highlight (`menu::list_step`). Leaving tells the action the end
//! (state -1). The game quitting ends it.
//!
//! Returns the answer (`list_answer`). Refused: a page whose every item cannot be chosen.
//! Called by `install::run_pages`.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{
    copy_picture, key_action, list_clear_marks, list_item_unusable, list_step, list_take_key, lock,
    page_action, run_frame, show, unlock,
};
use crate::list_handle::ListMenu;

const NO_CHOICE: BadInput = BadInput("a page whose every item cannot be chosen");

/// The list's action (+8) told where the page is.
fn tell(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
    list: ListMenu,
    state: u32,
) -> Result<(), Refused> {
    let action = list.action();
    if action != 0 {
        page_action(install, o, action, state, list)?;
    }
    Ok(())
}

/// The page of `list` run until it is left; its answer.
pub fn run_page(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
    list: ListMenu,
) -> Result<u32, Refused> {
    let list_state = &o.menus.list;
    use crate::install as i;
    let (strings, system) = (o.strings, o.system);
    list_state.list_answer.set(1);
    list_state.list_leave.set(0);
    list_state
        .move_clock
        .set(system.session.key_clock.ticks.get());
    list_state
        .key_clock
        .set(system.session.key_clock.ticks.get());
    list_state.choice.set(list.highlight());
    let count = list.count() as u32;
    let round = 2 * count as usize + 2;
    if count != 0 {
        list_clear_marks(list);
    }
    tell(install, o, list, 0)?;
    'page: {
        if list.flags() & 0xC0 != 0 {
            break 'page;
        }
        i::mark_page::mark_page(install, list_state, list);
        let mut steps = 0;
        while list_item_unusable(list_state, list) {
            steps += 1;
            if steps > round {
                return Err(NO_CHOICE.into());
            }
            list_state
                .choice
                .set(list_state.choice.get().wrapping_add(1));
            if count == list_state.choice.get() {
                list_state.choice.set(0);
            }
        }
        i::show_backdrop::show_backdrop(o)?;
        loop {
            i::lay_out_pieces::lay_out_pieces(install, list)?;
            i::measure_pieces::measure_pieces(
                crate::strings::append_string_by_id::StringOwners::of(o),
                install,
                strings,
                o.gfx,
            )?;
            i::place_pieces::place_pieces(install)?;
            run_frame(o, None)?;
            if lock(o.gfx, system) {
                copy_picture(o.gfx, system)?;
                i::draw_pieces::draw_pieces(
                    crate::strings::append_string_by_id::StringOwners::of(o),
                    install,
                    strings,
                    o.gfx,
                )?;
                unlock(o.gfx);
                show(o);
            }
            list_state
                .frame_clock
                .set(system.session.key_clock.ticks.get());
            list_state.choice_before.set(list_state.choice.get());
            // The key, as the callees leave it (`install::edit_key` changes it).
            let mut key = list_take_key(list_state, o.input)? as u32;
            if system.session.flags.quitting.is_set() {
                break 'page;
            }
            if key as u8 != 0 && key & 0x80 == 0 {
                let mut again = 0;
                loop {
                    let item = list.item(list_state.choice.get());
                    let (left, mut took) = i::page_key::page_key(o, install, key, item, list)?;
                    key = left;
                    if !took {
                        let keys = list.key_action();
                        if keys != 0 {
                            took = key_action(list_state, keys, key, list)?;
                        }
                    }
                    let moved = took || list_step(list_state, key, list.neighbours(), list)?;
                    if moved {
                        list_state.move_clock.set(list_state.frame_clock.get());
                    }
                    list_clear_marks(list);
                    tell(install, o, list, 1)?;
                    i::mark_page::mark_page(install, list_state, list);
                    if !list_item_unusable(list_state, list) {
                        break;
                    }
                    let arrow = matches!(key as u8, 0x66 | 0x6B | 0x68 | 0x69);
                    if arrow && list_state.choice.get() != list_state.choice_before.get() {
                        again += 1;
                        if again > round {
                            return Err(NO_CHOICE.into());
                        }
                        continue;
                    }
                    let mut steps = 0;
                    loop {
                        let k = key as u8;
                        if k == 0x66 || k == 0x68 {
                            let c = list_state.choice.get().wrapping_sub(1);
                            list_state.choice.set(c);
                            if (c as i32) < 0 {
                                list_state.choice.set(count.wrapping_sub(1));
                            }
                        } else {
                            let c = list_state.choice.get().wrapping_add(1);
                            list_state.choice.set(c);
                            if count == c {
                                list_state.choice.set(0);
                            }
                        }
                        if !list_item_unusable(list_state, list) {
                            break;
                        }
                        steps += 1;
                        if steps > round {
                            return Err(NO_CHOICE.into());
                        }
                    }
                    break;
                }
            } else {
                list_clear_marks(list);
                tell(install, o, list, 1)?;
                i::mark_page::mark_page(install, list_state, list);
                if list_item_unusable(list_state, list) {
                    let arrow = matches!(key as u8, 0x66 | 0x6B | 0x68 | 0x69);
                    if arrow && list_state.choice.get() != list_state.choice_before.get() {
                        return Err(BadInput("an arrow that is no key").into());
                    }
                    let mut steps = 0;
                    loop {
                        let c = list_state.choice.get().wrapping_add(1);
                        list_state.choice.set(c);
                        if count == c {
                            list_state.choice.set(0);
                        }
                        if !list_item_unusable(list_state, list) {
                            break;
                        }
                        steps += 1;
                        if steps > round {
                            return Err(NO_CHOICE.into());
                        }
                    }
                }
            }
            if list_state.list_leave.get() == 1 {
                break;
            }
        }
        i::show_backdrop::show_backdrop(o)?;
        tell(install, o, list, u32::MAX)?;
    }
    if count != 0 {
        list_clear_marks(list);
    }
    list.set_highlight(list_state.choice.get());
    Ok(list_state.list_answer.get() as u32)
}
