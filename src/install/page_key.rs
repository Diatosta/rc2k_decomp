//! A key on the page's highlighted item. Nothing for no key or one of 0x80 and above. An edit
//! being edited takes every key (`install::edit_key`); else the arrows (`install::key_up`,
//! `key_down`, `key_left`, `key_right`), 2 .. 0xA (`install::key_number`), Enter
//! (`install::item_enter`); Escape leaves the page with answer 0 unless the list's flags have
//! 0x10. Key 0x52 takes a screenshot with screenshots on. A key that did something sets
//! `key_taken`. Called by `install::run_page`.

use crate::errors::Refused;
use crate::list_handle::{ListItem, ListMenu};
use rc2k_core::install::items::{self, ItemKey, PageKey};

/// `key` (the key in its low byte, its bits in the next) on `item` of `list`. Returns the key
/// as the callees leave it (`install::edit_key` changes it) and whether the key did
/// something.
pub fn page_key(
    o: crate::owners::Owners<'static>,
    install: &'static crate::install::owner::Install,
    key: u32,
    item: ListItem,
    list: ListMenu,
) -> Result<(u32, bool), Refused> {
    use crate::install as i;
    let (gfx, system, list_state) = (o.gfx, o.system, &o.menus.list);
    let Some(way) = items::page_key(key as u8) else {
        // No key, or one of 0x80 and above.
        return Ok((key, key as u8 == 0));
    };
    let kind = item.kind();
    let edited = items::takes_every_key(kind, || item.edit_field().flags() & 1 != 0);
    let (key, plain_edit) = if edited {
        let outcome = i::edit_key::edit_key(
            crate::strings::append_string_by_id::StringOwners::of(o),
            o.input,
            install,
            key,
            item,
            gfx,
        )?;
        (outcome.key_word, outcome.plain_edit)
    } else {
        let taken = match way {
            PageKey::Item(ItemKey::Up) => Some(i::key_up::key_up(list_state, item)?),
            PageKey::Item(ItemKey::Down) => Some(i::key_down::key_down(list_state, item)?),
            PageKey::Item(ItemKey::Left) => Some(i::key_left::key_left(list_state, item)?),
            PageKey::Item(ItemKey::Right) => Some(i::key_right::key_right(list_state, item)?),
            PageKey::Item(ItemKey::Number) => {
                Some(i::key_number::key_number(list_state, key, item)?)
            }
            PageKey::Enter => Some(i::item_enter::item_enter(install, list_state, item)?),
            _ => None,
        };
        if let Some(taken) = taken {
            return Ok((key, took(list_state, taken)));
        }
        if way == PageKey::Escape {
            // The page is left unless the list stays.
            let stays = list.flags() & items::STAYS != 0;
            if !stays {
                list_state.list_answer.set(0);
                list_state.list_leave.set(1);
            }
            return Ok((key, !stays));
        }
        // Another key.
        (key, false)
    };
    if crate::gfx::shots::on_byte(gfx) != 0 && key as u8 == items::SCREENSHOT_KEY {
        crate::gfx::take_screenshot::take_screenshot(gfx, system)?;
    }
    Ok((key, took(list_state, plain_edit)))
}

/// `jnz` out, else the key marked as taken (zero set).
fn took(list_state: &crate::menu::list_state::ListState, zero: bool) -> bool {
    if zero {
        list_state.key_taken.set(1);
    }
    zero
}
