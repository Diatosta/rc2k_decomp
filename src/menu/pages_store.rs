//! The front end's list pages and its yes / no answer, out of RAL.EXE's image (IDA 0x75523C ..
//! 0x755258): the pages the save, scores, set-up and replay lists are on, the answer
//! the yes / no screens leave, and the two championship switches the event sets. One static.

use crate::session::PlainDword;

/// The list pages and the answer (IDA 0x75523C .. 0x755258).
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
