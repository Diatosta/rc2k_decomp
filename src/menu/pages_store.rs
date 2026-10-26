//! The front end's list pages and its yes / no answer, out of RAL.EXE's image (IDA 0x75523C ..
//! 0x755258; phase 7, typed): the pages the save, scores, set-up and replay lists are on, the answer
//! the yes / no screens leave, and the two championship switches the event sets. A `#[repr(C)]` struct
//! with the image's layout, one static; `moved::MOVED` points the range at it. (Split 2026-10-25 from
//! the front end's block `menu::store`.)

use crate::session::Dword;

/// The list pages and the answer (IDA 0x75523C .. 0x755258).
#[repr(C)]
pub struct MenuPages {
    /// `g_savePage`: The page the list of saved games is on (`menu::item_actions`, `menu::name_lists`).
    pub save_page: Dword,
    /// `dword_755240`: The page of the best scores.
    pub scores_page: Dword,
    /// `dword_755244`: The page of the set-up slots (`menu::load_setup_slot`).
    pub setup_slot_page: Dword,
    /// `dword_755248`: The page of the replays (`menu::name_lists`, `menu::player_actions`).
    pub replay_page: Dword,
    /// `g_yesNoAnswer`: The answer of a yes / no screen: 1 = yes (at start), 0 = no.
    pub yes_no_answer: Dword,
    /// `g_resetTimes`: 1 resets the players' times at the event's start (`menu::run_championship`, `saves::load_savegame` clears it).
    pub reset_times: Dword,
    /// `dword_755254`: Cleared by `menu::begin_event`; `menu::run_championship` counts the services in it.
    pub services: Dword,
}

/// First values the image's (`tests::menu_pages_starts_as_the_games`).
pub static MENU_PAGES: MenuPages = MenuPages {
    save_page: Dword::new(0x0),
    scores_page: Dword::new(0x0),
    setup_slot_page: Dword::new(0x0),
    replay_page: Dword::new(0x0),
    yes_no_answer: Dword::new(0x1),
    reset_times: Dword::new(0x0),
    services: Dword::new(0x0),
};

/// IDA 0x75523C, the first byte of the piece.
pub const START: usize = 0x75523C;
/// IDA 0x755258, the end of the piece.
pub const END: usize = 0x755258;

/// Where IDA 0x75523C is now (`moved::MOVED`'s home).
pub fn menu_pages_addr() -> usize {
    crate::pointers::host_number(&MENU_PAGES as *const MenuPages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn menu_pages_starts_as_the_games() {
        assert!(
            END - START <= size_of::<MenuPages>()
                && size_of::<MenuPages>() <= (END - START).next_multiple_of(4)
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(menu_pages_addr()) as *const u8,
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(MenuPages, save_page), 0x75523C);
        at(offset_of!(MenuPages, scores_page), 0x755240);
        at(offset_of!(MenuPages, setup_slot_page), 0x755244);
        at(offset_of!(MenuPages, replay_page), 0x755248);
        at(offset_of!(MenuPages, yes_no_answer), 0x75524C);
        at(offset_of!(MenuPages, reset_times), 0x755250);
        at(offset_of!(MenuPages, services), 0x755254);
    }
}
