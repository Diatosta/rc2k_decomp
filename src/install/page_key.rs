//! `install::page_key` (0x4146E4): a key on the page's highlighted item. Nothing for no key
//! or one of 0x80 and above . An edit being edited takes every key
//! (`install::edit_key`); else 0x66, 0x6B, 0x68, 0x69 (`install::key_up`, `key_down`,
//! `key_left`, `key_right`), 2 .. 0xA (`install::key_number`), Enter (`install::item_enter`);
//! Escape leaves the page with answer 0 unless the list's flags have 0x10. After an edit's
//! key or any other key: 0x52 a screenshot (`gfx::take_screenshot`) with screenshots on.
//! Then a key that did something (zero set) sets `key_taken` and gives zero set.
//!
//! Takes the key, item, list.
//! Caller `install::run_page`, reading zero.

use crate::errors::Refused;
use crate::menu::list_handle::{ListItem, ListMenu};
use rc2k_core::install::items::{self, ItemKey, PageKey};

/// `key` (the key in its low byte, its bits in the next) on `item` of `list`. Returns the key
/// as the callees leave it (`install::edit_key` changes it) and whether the key did
/// something.
pub fn page_key(
    install: &'static crate::install::owner::Install,
    gfx: &crate::gfx::owner::Gfx,
    key: u32,
    item: ListItem,
    list: ListMenu,
) -> Result<(u32, bool), Refused> {
    use crate::install as i;
    // Converted: `rc2k_core::install::items` (`page_key`, `takes_every_key`, `STAYS`,
    // `SCREENSHOT_KEY`).
    let Some(way) = items::page_key(key as u8) else {
        // No key, or one of 0x80 and above.
        return Ok((key, key as u8 == 0));
    };
    let kind = item.kind();
    let edited = items::takes_every_key(kind, || item.edit_field().flags() & 1 != 0);
    let (key, plain_edit) = if edited {
        let outcome = i::edit_key::edit_key(install, key, item)?;
        (outcome.key_word, outcome.plain_edit)
    } else {
        let taken = match way {
            PageKey::Item(ItemKey::Up) => Some(i::key_up::key_up(install, item)?),
            PageKey::Item(ItemKey::Down) => Some(i::key_down::key_down(install, item)?),
            PageKey::Item(ItemKey::Left) => Some(i::key_left::key_left(install, item)?),
            PageKey::Item(ItemKey::Right) => Some(i::key_right::key_right(install, item)?),
            PageKey::Item(ItemKey::Number) => Some(i::key_number::key_number(install, key, item)?),
            PageKey::Enter => Some(i::item_enter::item_enter(install, item)?),
            _ => None,
        };
        if let Some(taken) = taken {
            return Ok((key, took(install, taken)));
        }
        if way == PageKey::Escape {
            // The page is left unless the list stays.
            let stays = list.flags() & items::STAYS != 0;
            if !stays {
                install.pages.list_answer.set(0);
                install.pages.list_leave.set(1);
            }
            return Ok((key, !stays));
        }
        // Another key.
        (key, false)
    };
    if crate::gfx::shots::on_byte(gfx) != 0 && key as u8 == items::SCREENSHOT_KEY {
        crate::gfx::take_screenshot::take_screenshot(gfx)?;
    }
    Ok((key, took(install, plain_edit)))
}

/// `jnz` out, else the key marked as taken (zero set).
fn took(install: &'static crate::install::owner::Install, zero: bool) -> bool {
    if zero {
        install.pages.key_taken.set(1);
    }
    zero
}
