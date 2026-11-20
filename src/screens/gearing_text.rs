//! String 0x21F plus the current player's `gearing`, as a text of the car choice;
//! `flow.shown_gearing` is set to it. Refuses a current player past four and a text longer
//! than its place (0x100).

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::player::{Player, player};
use crate::screens::car_caption::put_string;

pub const TEXT_LEN: usize = 0x100;
const FIRST_STRING: u32 = 0x21F;

pub fn current(menus: &Menus) -> Result<&'_ crate::memory::session::Shared<Player>, Refused> {
    Ok(player(menus, menus.screens.event.current_player.get())
        .ok_or(BadInput("a current player past the four"))?)
}

pub fn gearing_text(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    text: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), Refused> {
    let value = current(menus)?.get().gearing;
    put_string(so, FIRST_STRING.wrapping_add(value), &text.title, TEXT_LEN)?;
    menus.screens.flow.write().shown_gearing = value;
    Ok(())
}

/// `menu::next_gearing` / `menu::previous_gearing`: `ScreenFlow.shown_gearing` on (past 2 to 0) or
/// back (below 0 to 2, both signed), the current player's `gearing` it, and its text.
pub fn step_gearing(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    text: &crate::screens::text_buffers_store::TextBuffers,
    on: bool,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    let p = current(menus)?;
    let kept = screens.flow.read().shown_gearing as i32;
    // The logic is in `rc2k_core::screens::captions::step_gearing`.
    let value = rc2k_core::screens::captions::step_gearing(kept, on);
    put_string(so, FIRST_STRING.wrapping_add(value), &text.title, TEXT_LEN)?;
    screens.flow.write().shown_gearing = value;
    p.update(|p| p.gearing = value);
    Ok(())
}
