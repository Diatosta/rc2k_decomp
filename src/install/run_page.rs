//! One of the installer's pages (a list menu) run until it is left: the page is told the start
//! (state 0), the items are marked, the highlight moved past items that cannot be chosen, the
//! backdrop shown; then each frame: the pieces laid out, measured, placed, the page drawn, a key
//! taken. A key goes to the item (`install::page_key`); if it did nothing, to the page's own
//! keys; if that did nothing either, it moves the highlight (`menu::list_step`). Leaving tells
//! the page the end (state -1). The game quitting ends it.
//!
//! Returns the answer (`list_answer`). Refused: a page whose every item cannot be chosen.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{copy_picture, list_take_key, lock, run_frame, show, unlock};
use rc2k_core::install::flow::Page;
use rc2k_core::install::model::{self, Kind, List, PageDef};

const NO_CHOICE: BadInput = BadInput("a page whose every item cannot be chosen");

const PAST_ITEMS: BadInput = BadInput("an item past the items of a list");

/// Whether the highlighted item cannot be chosen.
fn unusable(
    list_state: &crate::menu::list_state::ListState,
    def: &PageDef,
) -> Result<bool, BadInput> {
    def.items
        .get(list_state.choice.get() as usize)
        .map(model::Item::unusable)
        .ok_or(PAST_ITEMS)
}

/// The page's marks made again: those its facts give, then the highlight lit.
fn mark(
    list_state: &crate::menu::list_state::ListState,
    facts: &model::Facts,
    def: &mut PageDef,
) -> Result<(), Refused> {
    def.clear_marks();
    def.apply_marks(facts)?;
    mark_highlight(list_state, def);
    Ok(())
}

/// The highlighted item lit (by the blink clock) unless it has marks.
fn mark_highlight(list_state: &crate::menu::list_state::ListState, def: &mut PageDef) {
    let clock = list_state
        .frame_clock
        .get()
        .wrapping_sub(list_state.move_clock.get());
    def.mark_highlight(list_state.choice.get(), clock);
}

/// `key` moving the highlight of a page of `count` items (a neighbours table the pages have
/// none of); whether it moved.
fn step_highlight(list_state: &crate::menu::list_state::ListState, key: u32, count: u8) -> bool {
    use rc2k_core::menu::list_menu::{MOVES, Step, step};
    let key = key as u8;
    if !MOVES.contains(&key) {
        return false;
    }
    let choice = list_state.choice.get();
    let Step::Moved(item) = step(key, choice as u8, count, false, |_| 0) else {
        return false;
    };
    // Only the low byte is the item; the rest is kept.
    list_state.choice.set(choice & !0xFF | item as u32);
    true
}

/// The Enter key.
#[cfg(touch_screen)]
const ENTER: u32 = 0x1C;

/// The item (its number in the list) a tap on the page since the last frame landed on, if it can
/// be chosen: the piece the tap is in belongs to it.
#[cfg(touch_screen)]
fn tapped_item(def: &PageDef, laid: &[crate::install::lay_out_pieces::Piece]) -> Option<u32> {
    use crate::install::lay_out_pieces::Piece;
    use rc2k_core::install::model::Owner;
    let (x, y) = crate::shell::touch_menu::take_installer_tap()?;
    let inside = |piece: &Piece| {
        let (px, py) = (piece.x as i32 as f32, piece.y as i32 as f32);
        let width = piece.width as f32;
        // A piece's place is where its text starts or where it is centred, by how the page lays
        // it out: both are taken, from half a width before the place to a width after it.
        x >= px - width / 2.0 && x < px + width && y >= py && y < py + piece.height as f32
    };
    let piece = laid.iter().find(|piece| inside(piece))?;
    // The title's piece belongs to no item.
    let Some(Owner::Item(found)) = piece.item else {
        return None;
    };
    let usable = def
        .items
        .get(found as usize)
        .is_some_and(|item| !item.unusable());
    if usable {
        crate::shell::touch_menu::tap_used();
    }
    usable.then_some(found)
}

/// `page` run until it is left; its answer.
pub fn run_page(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
    page: Page,
) -> Result<u32, Refused> {
    let list_state = &o.menus.list;
    use crate::install as i;
    let (strings, system) = (o.strings, o.system);
    let facts = i::page_state::facts(install);
    let list = List::of(page, facts.drives);
    let mut def = model::build(list, &facts)?;
    if page == Page::Folder {
        // The folder typed is the edit's text while the page is shown.
        if let Some(Kind::Edit(edit)) = def.items.get_mut(1).map(|item| &mut item.kind) {
            edit.text = i::page_state::typed_text(install);
        }
    }
    list_state.list_answer.set(1);
    list_state.list_leave.set(0);
    list_state
        .move_clock
        .set(system.session.key_clock.ticks.get());
    list_state
        .key_clock
        .set(system.session.key_clock.ticks.get());
    list_state.choice.set(install.highlights.get(list));
    let count = def.items.len() as u32;
    let round = 2 * count as usize + 2;
    i::page_actions::start(install, list_state, page, &mut def);
    // A tap on the page is for its items (`shell::touch_menu`).
    #[cfg(touch_screen)]
    let _taps = crate::shell::touch_menu::InstallerPage::begin();
    'page: {
        mark_highlight(list_state, &mut def);
        let mut steps = 0;
        while unusable(list_state, &def)? {
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
            let mut laid = i::lay_out_pieces::lay_out_pieces(install, &def)?;
            i::measure_pieces::measure_pieces(
                crate::strings::append_string_by_id::StringOwners::of(o),
                install,
                &def,
                &mut laid,
                strings,
                o.gfx,
            )?;
            i::place_pieces::place_pieces(&def, &mut laid)?;
            run_frame(o, None)?;
            if lock(o.gfx, system) {
                copy_picture(o.gfx, system)?;
                i::draw_pieces::draw_pieces(
                    crate::strings::append_string_by_id::StringOwners::of(o),
                    install,
                    &def,
                    &laid,
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
            // The key, as the callees leave it (an edit changes it).
            let mut key = list_take_key(list_state, o.input)? as u32;
            if system.session.flags.quitting.is_set() {
                break 'page;
            }
            // A tap on an item highlights it and chooses it, as Enter does.
            #[cfg(touch_screen)]
            if let Some(chosen) = tapped_item(&def, &laid) {
                list_state.choice.set(chosen);
                key = ENTER;
            }
            i::page_key::type_into_edit(o, install, &mut def)?;
            if key as u8 != 0 && key & 0x80 == 0 {
                let mut again = 0;
                loop {
                    let (left, took) = i::page_key::page_key(o, install, page, &mut def, key)?;
                    key = left;
                    let moved = took || step_highlight(list_state, key, count as u8);
                    if moved {
                        list_state.move_clock.set(list_state.frame_clock.get());
                    }
                    mark(list_state, &facts, &mut def)?;
                    if !unusable(list_state, &def)? {
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
                        if !unusable(list_state, &def)? {
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
                mark(list_state, &facts, &mut def)?;
                if unusable(list_state, &def)? {
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
                        if !unusable(list_state, &def)? {
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
        i::page_actions::end(install, o, page)?;
    }
    if page == Page::Folder
        && let Some(Kind::Edit(edit)) = def.items.get(1).map(|item| &item.kind)
    {
        i::page_state::store_typed_text(install, &edit.text);
    }
    install.highlights.set(list, list_state.choice.get());
    Ok(u32::from(list_state.list_answer.get()))
}
