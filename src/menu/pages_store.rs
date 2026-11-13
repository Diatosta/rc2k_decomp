//! The front end's list pages and its yes / no answer: the pages the save, scores, set-up and
//! replay lists are on, the answer the yes / no screens leave, and the two championship
//! switches the event sets. One static.

use crate::session::PlainDword;

/// The list pages and the answer.
#[derive(rc2k_derive::Cells)]
pub struct MenuPages {
    /// The page the list of saved games is on (`menu::item_actions`, `menu::name_lists`).
    pub save_page: PlainDword,
    /// The page of the best scores.
    pub scores_page: PlainDword,
    /// The page of the set-up slots (`menu::load_setup_slot`).
    pub setup_slot_page: PlainDword,
    /// The page of the replays (`menu::name_lists`, `menu::player_actions`).
    pub replay_page: PlainDword,
    /// The answer of a yes / no screen: 1 = yes (at start), 0 = no.
    pub yes_no_answer: PlainDword,
    /// 1 resets the players' times at the event's start (`menu::run_championship`,
    /// `saves::load_savegame` clears it).
    pub reset_times: PlainDword,
    /// Cleared by `menu::begin_event`; `menu::run_championship` counts the services in it.
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
