//! `screens::gearing_text` (0x4699E0): string 0x21F + the current player's `gearing`, as
//! a text of the car choice.
//!
//!
//! `flow.shown_gearing` = the current player's (`event.current_player`) `gearing`; the string and a NUL into
//! the title buffer. In two entry lists (0x7593FE, 0x75941A). Refuses a current player past four, a
//! text longer than its place (0x100). The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::player::{Player, player};
use crate::screens::car_caption::put_string;

pub const TEXT_LEN: usize = 0x100;
const FIRST_STRING: u32 = 0x21F;

const _: () = assert!(std::mem::offset_of!(Player, gearing) == 0x30);

pub fn current(menus: &Menus) -> Result<&'_ crate::session::Shared<Player>, Refused> {
    Ok(player(menus, menus.screens.event.current_player.get())
        .ok_or(BadInput("a current player past the four"))?)
}

pub fn gearing_text(menus: &Menus) -> Result<(), Refused> {
    let value = current(menus)?.get().gearing;
    put_string(
        FIRST_STRING.wrapping_add(value),
        &crate::screens::text_buffers_store::TEXT_BUFFERS.title,
        TEXT_LEN,
    )?;
    menus.screens.flow.shown_gearing.set(value);
    Ok(())
}

/// `menu::next_gearing` / `menu::previous_gearing`: `dword_7554F0` on (past 2 to 0) or back
/// (below 0 to 2, both signed), the current player's `gearing` it, and its text.
pub fn step_gearing(menus: &Menus, on: bool) -> Result<(), Refused> {
    let screens = &menus.screens;
    let p = current(menus)?;
    let kept = screens.flow.shown_gearing.get() as i32;
    // Converted: `rc2k_core::screens::captions::step_gearing`.
    let value = rc2k_core::screens::captions::step_gearing(kept, on);
    put_string(
        FIRST_STRING.wrapping_add(value),
        &crate::screens::text_buffers_store::TEXT_BUFFERS.title,
        TEXT_LEN,
    )?;
    screens.flow.shown_gearing.set(value);
    p.update(|p| p.gearing = value);
    Ok(())
}
