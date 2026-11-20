//! `menu::find_section_player`: the first of the four players whose overall points for the section
//! (`overall_points[section]`) are 0x14 (a reading: done with it); none: index -1.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

/// What `find_section_player` finds: the player's index (`u32::MAX` for none) and its record
/// (player 4's for none).
#[derive(Clone, Copy)]
pub struct SectionPlayer<'a> {
    pub index: u32,
    pub record: PlayerCell<'a>,
}

pub fn find_section_player(menus: &Menus) -> Result<SectionPlayer<'_>, Refused> {
    let screens = &menus.screens;
    let section: u32 = screens.event.read().section;
    let done: [u32; 4] =
        std::array::from_fn(|n| player(menus, n as u32).get().overall_points[section as usize]);
    let found = rc2k_core::menu::points::section_player(done);
    Ok(SectionPlayer {
        index: found.unwrap_or(u32::MAX),
        record: player(menus, found.unwrap_or(3)),
    })
}
