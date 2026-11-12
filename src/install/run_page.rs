//! `install::run_page` (0x4144CF): one of the installer's pages (a list menu) run until it
//! is left: the answer 1, not leaving, the clocks, the highlight from the list's +4; the
//! marks cleared (`menu::list_clear_marks`) and the list's action (+8) told the start (state
//! 0). A list with 0x40 or 0x80 is only that action. Else the items marked
//! (`install::mark_page`), the highlight moved on past items that cannot be chosen
//! (`menu::list_item_unusable`), the backdrop shown (`install::show_backdrop`); then each
//! frame: the pieces laid out, measured, placed, a frame's messages (`screens::run_frame`,
//! no key), the page drawn (lock, backdrop, `install::draw_pieces`, unlock, show), the
//! clocks, a key (`menu::list_take_key`). The game quitting ends it. A key (1 .. 0x7F)
//! goes to the item (`install::page_key`); if it did nothing, to the list's key action
//! (+0x14); if that did nothing either, it moves the highlight (`menu::list_step`, the
//! neighbours at +0x10); what took the key restarts the move clock. Then the marks again,
//! the action told (state 1), the items marked, and a highlight that cannot be chosen moved
//! on (an arrow on to the same item as before: the key again on the new one; else back for
//! 0x66 and 0x68, on for the rest, round the list). Leaving (`g_listLeave`): the backdrop,
//! the action told the end (state -1). At the end the marks cleared and the highlight kept.
//!
//! Takes the list. Returns the answer (`list_answer`).
//! A page whose every item cannot be chosen would go round for ever: refused. Caller
//! `install::run_pages`.
//! The quitting way is not tested (it would set the game's `g_quitting`).

use crate::errors::{BadInput, Refused};
use crate::install::installer::{
    copy_picture, key_action, list_clear_marks, list_item_unusable, list_step, list_take_key, lock,
    page_action, run_frame, show, unlock,
};
use crate::menu::list_handle::ListMenu;

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
    use crate::install as i;
    let (strings, system, gfx) = (o.strings, o.system, o.gfx);
    install.pages.list_answer.set(1);
    install.pages.list_leave.set(0);
    install
        .pages
        .move_clock
        .set(system.session.clocks.key_ticks.get());
    install
        .pages
        .key_clock
        .set(system.session.clocks.key_ticks.get());
    install.pages.choice.set(list.highlight());
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
        i::mark_page::mark_page(install, list);
        let mut steps = 0;
        while list_item_unusable(list) {
            steps += 1;
            if steps > round {
                return Err(NO_CHOICE.into());
            }
            install
                .pages
                .choice
                .set(install.pages.choice.get().wrapping_add(1));
            if count == install.pages.choice.get() {
                install.pages.choice.set(0);
            }
        }
        i::show_backdrop::show_backdrop(system)?;
        loop {
            i::lay_out_pieces::lay_out_pieces(install, list)?;
            i::measure_pieces::measure_pieces(install, strings)?;
            i::place_pieces::place_pieces(install)?;
            run_frame(system, None)?;
            if lock() {
                copy_picture()?;
                i::draw_pieces::draw_pieces(install, strings)?;
                unlock();
                show();
            }
            install
                .pages
                .frame_clock
                .set(system.session.clocks.key_ticks.get());
            install.pages.choice_before.set(install.pages.choice.get());
            // The key, as the callees leave it (`install::edit_key` changes it).
            let mut key = list_take_key()? as u32;
            if system.session.flags.quitting.is_set() {
                break 'page;
            }
            if key as u8 != 0 && key & 0x80 == 0 {
                let mut again = 0;
                loop {
                    let item = list.item(install.pages.choice.get());
                    let (left, mut took) = i::page_key::page_key(install, gfx, key, item, list)?;
                    key = left;
                    if !took {
                        let keys = list.key_action();
                        if keys != 0 {
                            took = key_action(install, keys, key, list)?;
                        }
                    }
                    let moved = took || list_step(key, list.neighbours(), list)?;
                    if moved {
                        install
                            .pages
                            .move_clock
                            .set(install.pages.frame_clock.get());
                    }
                    list_clear_marks(list);
                    tell(install, o, list, 1)?;
                    i::mark_page::mark_page(install, list);
                    if !list_item_unusable(list) {
                        break;
                    }
                    let arrow = matches!(key as u8, 0x66 | 0x6B | 0x68 | 0x69);
                    if arrow && install.pages.choice.get() != install.pages.choice_before.get() {
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
                            let c = install.pages.choice.get().wrapping_sub(1);
                            install.pages.choice.set(c);
                            if (c as i32) < 0 {
                                install.pages.choice.set(count.wrapping_sub(1));
                            }
                        } else {
                            let c = install.pages.choice.get().wrapping_add(1);
                            install.pages.choice.set(c);
                            if count == c {
                                install.pages.choice.set(0);
                            }
                        }
                        if !list_item_unusable(list) {
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
                i::mark_page::mark_page(install, list);
                if list_item_unusable(list) {
                    let arrow = matches!(key as u8, 0x66 | 0x6B | 0x68 | 0x69);
                    if arrow && install.pages.choice.get() != install.pages.choice_before.get() {
                        return Err(BadInput("an arrow that is no key").into());
                    }
                    let mut steps = 0;
                    loop {
                        let c = install.pages.choice.get().wrapping_add(1);
                        install.pages.choice.set(c);
                        if count == c {
                            install.pages.choice.set(0);
                        }
                        if !list_item_unusable(list) {
                            break;
                        }
                        steps += 1;
                        if steps > round {
                            return Err(NO_CHOICE.into());
                        }
                    }
                }
            }
            if install.pages.list_leave.get() == 1 {
                break;
            }
        }
        i::show_backdrop::show_backdrop(system)?;
        tell(install, o, list, u32::MAX)?;
    }
    if count != 0 {
        list_clear_marks(list);
    }
    list.set_highlight(install.pages.choice.get());
    Ok(install.pages.list_answer.get() as u32)
}
