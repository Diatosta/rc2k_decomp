//! `install::run_page` (0x4144CF): one of the installer's pages (a list menu) run until it
//! is left: the answer 1, not leaving, the clocks, the highlight from the list's +4; the
//! marks cleared (`menu::list_clear_marks`) and the list's action (+8) told the start (ebx
//! 0). A list with 0x40 or 0x80 is only that action. Else the items marked
//! (`install::mark_page`), the highlight moved on past items that cannot be chosen
//! (`menu::list_item_unusable`), the backdrop shown (`install::show_backdrop`); then each
//! frame: the pieces laid out, measured, placed, a frame's messages (`screens::run_frame`,
//! eax 0), the page drawn (lock, backdrop, `install::draw_pieces`, unlock, show), the
//! clocks, a key (`menu::list_take_key`). The game quitting ends it. A key (al 1 .. 0x7F)
//! goes to the item (`install::page_key`); if it did nothing, to the list's key action
//! (+0x14); if that did nothing either, it moves the highlight (`menu::list_step`, the
//! neighbours at +0x10); what took the key restarts the move clock. Then the marks again,
//! the action told (ebx 1), the items marked, and a highlight that cannot be chosen moved
//! on (an arrow on to the same item as before: the key again on the new one; else back for
//! 0x66 and 0x68, on for the rest, round the list). Leaving (`g_listLeave`): the backdrop,
//! the action told the end (ebx -1). At the end the marks cleared and the highlight kept.
//!
//! In:  edi the list
//! Out: eax the answer (`byte_51D415`); esi, edi, ebp kept; ebx, ecx, edx as left
//! A page whose every item cannot be chosen would go round for ever: refused. Caller
//! `install::run_pages`.
//! The quitting way is not tested (it would set the game's `g_quitting`).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{
    copy_picture, key_action, list_clear_marks, list_item_unusable, list_step, list_take_key, lock,
    page_action, rb, rd, run_frame, show, unlock, wd,
};
use crate::install::vars::INSTALLER_PAGES;
use crate::session::CLOCKS;
use crate::session::FLAGS;

const NO_CHOICE: BadInput = BadInput("a page whose every item cannot be chosen");

/// The list's action (+8) told where the page is.
fn tell(list: u32, state: u32) -> Result<(), Refused> {
    let action = rd(list + 8);
    if action != 0 {
        page_action(action, state, list)?;
    }
    Ok(())
}

/// The page of `list` run until it is left; its answer.
pub fn run_page(list: u32) -> Result<u32, Refused> {
    use crate::install as i;
    INSTALLER_PAGES.list_answer.set(1);
    INSTALLER_PAGES.list_leave.set(0);
    INSTALLER_PAGES.move_clock.set(CLOCKS.key_ticks.get());
    INSTALLER_PAGES.key_clock.set(CLOCKS.key_ticks.get());
    INSTALLER_PAGES.choice.set(rd(list + 4));
    let count = rb(list + 1) as u32;
    let round = 2 * count as usize + 2;
    if count != 0 {
        list_clear_marks(list);
    }
    tell(list, 0)?;
    'page: {
        if rb(list) & 0xC0 != 0 {
            break 'page;
        }
        i::mark_page::mark_page(list);
        let mut steps = 0;
        while list_item_unusable(list) {
            steps += 1;
            if steps > round {
                return Err(NO_CHOICE.into());
            }
            INSTALLER_PAGES
                .choice
                .set(INSTALLER_PAGES.choice.get().wrapping_add(1));
            if count == INSTALLER_PAGES.choice.get() {
                INSTALLER_PAGES.choice.set(0);
            }
        }
        i::show_backdrop::show_backdrop()?;
        loop {
            i::lay_out_pieces::lay_out_pieces(list)?;
            i::measure_pieces::measure_pieces()?;
            i::place_pieces::place_pieces()?;
            run_frame(None)?;
            if lock() {
                copy_picture()?;
                i::draw_pieces::draw_pieces()?;
                unlock();
                show();
            }
            INSTALLER_PAGES.frame_clock.set(CLOCKS.key_ticks.get());
            INSTALLER_PAGES
                .choice_before
                .set(INSTALLER_PAGES.choice.get());
            // eax: the key, as the callees leave it (`install::edit_key` changes it)
            let mut key = list_take_key()? as u32;
            if FLAGS.quitting.get() == 1 {
                break 'page;
            }
            if key as u8 != 0 && key & 0x80 == 0 {
                let mut again = 0;
                loop {
                    let item = list + 0x18 + 0x10 * INSTALLER_PAGES.choice.get();
                    let (left, mut took) = i::page_key::page_key(key, item, list)?;
                    key = left;
                    if !took {
                        let keys = rd(list + 0x14);
                        if keys != 0 {
                            took = key_action(keys, key, list)?;
                        }
                    }
                    let moved = took || list_step(key, rd(list + 0x10), list);
                    if moved {
                        INSTALLER_PAGES
                            .move_clock
                            .set(INSTALLER_PAGES.frame_clock.get());
                    }
                    list_clear_marks(list);
                    tell(list, 1)?;
                    i::mark_page::mark_page(list);
                    if !list_item_unusable(list) {
                        break;
                    }
                    let arrow = matches!(key as u8, 0x66 | 0x6B | 0x68 | 0x69);
                    if arrow && INSTALLER_PAGES.choice.get() != INSTALLER_PAGES.choice_before.get()
                    {
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
                            let c = INSTALLER_PAGES.choice.get().wrapping_sub(1);
                            INSTALLER_PAGES.choice.set(c);
                            if (c as i32) < 0 {
                                INSTALLER_PAGES.choice.set(count.wrapping_sub(1));
                            }
                        } else {
                            let c = INSTALLER_PAGES.choice.get().wrapping_add(1);
                            INSTALLER_PAGES.choice.set(c);
                            if count == c {
                                INSTALLER_PAGES.choice.set(0);
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
                tell(list, 1)?;
                i::mark_page::mark_page(list);
                if list_item_unusable(list) {
                    let arrow = matches!(key as u8, 0x66 | 0x6B | 0x68 | 0x69);
                    if arrow && INSTALLER_PAGES.choice.get() != INSTALLER_PAGES.choice_before.get()
                    {
                        return Err(BadInput("an arrow that is no key").into());
                    }
                    let mut steps = 0;
                    loop {
                        let c = INSTALLER_PAGES.choice.get().wrapping_add(1);
                        INSTALLER_PAGES.choice.set(c);
                        if count == c {
                            INSTALLER_PAGES.choice.set(0);
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
            if INSTALLER_PAGES.list_leave.get() == 1 {
                break;
            }
        }
        i::show_backdrop::show_backdrop()?;
        tell(list, u32::MAX)?;
    }
    if count != 0 {
        list_clear_marks(list);
    }
    wd(list + 4, INSTALLER_PAGES.choice.get());
    Ok(INSTALLER_PAGES.list_answer.get() as u32)
}
