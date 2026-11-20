//! A key on the page's highlighted item (`rc2k_core::install::model::PageDef::key`), and what it
//! asks of the installer: a key that did something is noted, the answer a key leaves the page with
//! is set, the lack of room is ignored, the screenshot key takes a screenshot. Then the page's own
//! key, if the item took none.

use crate::errors::{BadInput, Refused};
use crate::install::page_state::FolderHost;
use rc2k_core::install::flow::Page;
use rc2k_core::install::items::SCREENSHOT_KEY;
use rc2k_core::install::model::{self, Kind, PageDef};

/// The host of the folder field's keys, for the page's edit item (the folder's).
fn host(
    o: crate::owners::Owners<'static>,
    install: &'static crate::install::owner::Install,
    def: &PageDef,
) -> FolderHost {
    let (font_kind, most) = def
        .items
        .iter()
        .find_map(|item| match &item.kind {
            Kind::Edit(edit) => Some((edit.font_kind, edit.most)),
            _ => None,
        })
        .unwrap_or_default();
    FolderHost {
        so: crate::strings::append_string_by_id::StringOwners::of(o),
        install,
        gfx: o.gfx,
        font_kind,
        most,
    }
}

/// The page left with `answer`.
fn leave(list_state: &crate::menu::list_state::ListState, answer: u8) {
    list_state.list_answer.set(answer);
    list_state.list_leave.set(1);
}

/// `key` (the key in its low byte, its bits in the next) on the highlighted item. Returns the key
/// as the edit leaves it (it changes it) and whether the key did something.
pub(super) fn page_key(
    o: crate::owners::Owners<'static>,
    install: &'static crate::install::owner::Install,
    page: Page,
    def: &mut PageDef,
    key: u32,
) -> Result<(u32, bool), Refused> {
    let list_state = &o.menus.list;
    let highlight = list_state.choice.get();
    if highlight as usize >= def.items.len() {
        return Err(BadInput("an item past the items of a list").into());
    }
    let keyed = def.key(highlight, key, &mut host(o, install, def))?;
    if keyed.noted {
        list_state.key_taken.set(1);
    }
    if let Some(answer) = keyed.leave {
        leave(list_state, answer);
    }
    if keyed.ignore_room {
        install.flags.ignore_room.set(true);
    }
    if keyed.other
        && crate::gfx::shots::on_byte(o.gfx) != 0
        && keyed.key_word as u8 == SCREENSHOT_KEY
    {
        crate::gfx::take_screenshot::take_screenshot(o.gfx, o.system)?;
    }
    let mut taken = keyed.taken;
    if !taken {
        // The page's own key.
        let (own, answer) = model::own_key(page, keyed.key_word);
        if let Some(answer) = answer {
            leave(list_state, answer);
        }
        taken = own;
    }
    Ok((keyed.key_word, taken))
}

/// The characters typed since the last frame go into the item if it is an edit being edited
/// (as keys with the "character itself" bit; the scan codes of printable keys are not read
/// meanwhile, see `input::typed_text`).
pub(super) fn type_into_edit(
    o: crate::owners::Owners<'static>,
    install: &'static crate::install::owner::Install,
    def: &mut PageDef,
) -> Result<(), Refused> {
    let highlight = o.menus.list.choice.get();
    let Some(item) = def.items.get(highlight as usize) else {
        return Err(BadInput("an item past the items of a list").into());
    };
    if !matches!(&item.kind, Kind::Edit(edit) if edit.editing) {
        return Ok(());
    }
    let queued = crate::input::typed_text::take();
    for &c in &queued.chars {
        let mut host = host(o, install, def);
        def.key(highlight, 0x100 | u32::from(c), &mut host)?;
        o.menus.list.key_taken.set(1);
    }
    Ok(())
}
