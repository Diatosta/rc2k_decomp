//! The front end's list pages and its yes / no answer, out of RAL.EXE's image (IDA 0x75523C ..
//! 0x755258): the pages the save, scores, set-up and replay lists are on, the answer
//! the yes / no screens leave, and the two championship switches the event sets. A `#[repr(C)]` struct
//! with the image's layout, one static; `moved::MOVED` points the range at it.

use crate::session::PlainDword;

/// The list pages and the answer (IDA 0x75523C .. 0x755258).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MenuPages {
    /// `g_savePage`: The page the list of saved games is on (`menu::item_actions`, `menu::name_lists`).
    pub save_page: PlainDword,
    /// `dword_755240`: The page of the best scores.
    pub scores_page: PlainDword,
    /// `dword_755244`: The page of the set-up slots (`menu::load_setup_slot`).
    pub setup_slot_page: PlainDword,
    /// `dword_755248`: The page of the replays (`menu::name_lists`, `menu::player_actions`).
    pub replay_page: PlainDword,
    /// `g_yesNoAnswer`: The answer of a yes / no screen: 1 = yes (at start), 0 = no.
    pub yes_no_answer: PlainDword,
    /// `g_resetTimes`: 1 resets the players' times at the event's start (`menu::run_championship`, `saves::load_savegame` clears it).
    pub reset_times: PlainDword,
    /// `dword_755254`: Cleared by `menu::begin_event`; `menu::run_championship` counts the services in it.
    pub services: PlainDword,
}

/// First values the image's (`tests::menu_pages_starts_as_the_games`).
impl MenuPages {
    pub const fn new() -> Self {
        MenuPages {
            save_page: PlainDword::new(0x0),
            scores_page: PlainDword::new(0x0),
            setup_slot_page: PlainDword::new(0x0),
            replay_page: PlainDword::new(0x0),
            yes_no_answer: PlainDword::new(0x1),
            reset_times: PlainDword::new(0x0),
            services: PlainDword::new(0x0),
        }
    }
}

/// IDA 0x75523C, the first byte of the piece.
pub const START: usize = 0x75523C;
/// IDA 0x755258, the end of the piece.
pub const END: usize = 0x755258;

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn menu_pages_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::menu::owner::menus().tables.pages,
        );
        assert_eq!(offset_of!(MenuPages, save_page), 0x0);
        assert_eq!(offset_of!(MenuPages, scores_page), 0x4);
        assert_eq!(offset_of!(MenuPages, setup_slot_page), 0x8);
        assert_eq!(offset_of!(MenuPages, replay_page), 0xc);
        assert_eq!(offset_of!(MenuPages, yes_no_answer), 0x10);
        assert_eq!(offset_of!(MenuPages, reset_times), 0x14);
        assert_eq!(offset_of!(MenuPages, services), 0x18);
    }
}
