//! One of the installer's pages (a list menu) run until it is left: the page is told the start
//! (state 0), the items are marked, the highlight moved past items that cannot be chosen, the
//! backdrop shown; then each frame: the pieces laid out, measured, placed, the page drawn, a key
//! taken. A key goes to the item (`install::page_key`); if it did nothing, to the page's own
//! keys; if that did nothing either, it moves the highlight (`menu::list_step`). Leaving tells
//! the page the end (state -1). The game quitting ends it.
//!
//! Returns the answer (`list_answer`). Refused: a page whose every item cannot be chosen.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{
    copy_picture, list_clear_marks, list_item_unusable, list_step, list_take_key, lock, run_frame,
    show, unlock,
};
use crate::memory::list_handle::ListMenu;
use rc2k_core::install::flow::Page;

const NO_CHOICE: BadInput = BadInput("a page whose every item cannot be chosen");

/// The page told where it is: `state` 0 at the start, 1 after a key, -1 at the end.
fn tell(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
    page: Page,
    list: ListMenu,
    state: u32,
) -> Result<(), Refused> {
    use crate::install as i;
    let list_state = &o.menus.list;
    match page {
        Page::Language => i::language_page::language_page(install, o, state, list)?,
        Page::Main => i::mark_main_page::mark_main_page(install, list),
        Page::Drive => i::drive_page::drive_page(install, list_state, state, list),
        Page::Folder => i::folder_page::folder_page(install, list_state, state, list),
        Page::NoRoom => i::room_page::room_page(install, state),
        Page::Removal => i::removal_page::removal_page(install, list_state, state),
    }
    Ok(())
}

/// The page's own keys (only the folder page has some): whether `key` was taken.
fn page_own_key(
    list_state: &crate::menu::list_state::ListState,
    page: Page,
    key: u32,
    list: ListMenu,
) -> bool {
    page == Page::Folder && crate::install::folder_key::folder_key(list_state, key, list)
}

/// The Enter key.
#[cfg(touch_screen)]
const ENTER: u32 = 0x1C;

/// The item (its number in the list) a tap on the page since the last frame landed on, if it can
/// be chosen: the piece the tap is in belongs to it.
#[cfg(touch_screen)]
fn tapped_item(
    install: &'static crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    list: ListMenu,
) -> Option<u32> {
    use crate::install::page_records::{Piece, piece_count};
    let (x, y) = crate::shell::touch_menu::take_installer_tap()?;
    let inside = |n: u32| {
        let piece = Piece::nth(install, n);
        let (px, py) = (piece.x() as i32 as f32, piece.y() as i32 as f32);
        let width = piece.width() as f32;
        // A piece's place is where its text starts or where it is centred, by how the page lays
        // it out: both are taken, from half a width before the place to a width after it.
        x >= px - width / 2.0 && x < px + width && y >= py && y < py + piece.height() as f32
    };
    let piece = (0..piece_count(install)).find(|&n| inside(n))?;
    let item = Piece::nth(install, piece).item(install);
    let before = list_state.choice.get();
    let found = (0..u32::from(list.count())).find(|&k| list.item(k) == item)?;
    list_state.choice.set(found);
    let usable = !list_item_unusable(list_state, list);
    list_state.choice.set(before);
    if usable {
        crate::shell::touch_menu::tap_used();
    }
    usable.then_some(found)
}

/// `page`, shown as `list`, run until it is left; its answer.
pub fn run_page(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
    page: Page,
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
        list_clear_marks(install, list);
    }
    tell(install, o, page, list, 0)?;
    // A tap on the page is for its items (`shell::touch_menu`).
    #[cfg(touch_screen)]
    let _taps = crate::shell::touch_menu::InstallerPage::begin();
    'page: {
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
            // A tap on an item highlights it and chooses it, as Enter does.
            #[cfg(touch_screen)]
            if let Some(chosen) = tapped_item(install, list_state, list) {
                list_state.choice.set(chosen);
                key = ENTER;
            }
            i::page_key::type_into_edit(o, install, list.item(list_state.choice.get()))?;
            if key as u8 != 0 && key & 0x80 == 0 {
                let mut again = 0;
                loop {
                    let item = list.item(list_state.choice.get());
                    let (left, mut took) = i::page_key::page_key(o, install, key, item, list)?;
                    key = left;
                    if !took {
                        took = page_own_key(list_state, page, key, list);
                    }
                    let moved = took || list_step(list_state, key, list.neighbours(), list)?;
                    if moved {
                        list_state.move_clock.set(list_state.frame_clock.get());
                    }
                    list_clear_marks(install, list);
                    tell(install, o, page, list, 1)?;
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
                list_clear_marks(install, list);
                tell(install, o, page, list, 1)?;
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
        tell(install, o, page, list, u32::MAX)?;
    }
    if count != 0 {
        list_clear_marks(install, list);
    }
    list.set_highlight(list_state.choice.get());
    Ok(list_state.list_answer.get() as u32)
}
