//! The screens' kept items, out of RAL.EXE's image (IDA 0x747660 .. 0x747998; phase 7,
//! typed): one slot per menu screen in two arrays, the item highlighted when the screen was
//! last left (`current`, the screen loop's `saved_item`) and the item some screens start
//! on (`first`, read into `measure_x` or the action's argument on entry). The screens name
//! their slot by `slot::*`; the 20 slots no ported screen names keep the image's values.
//! `#[repr(C)]`, the image's layout; `moved::MOVED` points the range at it.

use crate::session::Dword;

/// The screens' slots (an index into both arrays), by the screen that uses it.
pub mod slot {
    pub const SELECT_STAGE: usize = 0;
    pub const TIME_TRIAL_SELECT_STAGE: usize = 1;
    pub const ARCADE_NEXT_STAGE: usize = 2;
    pub const ARCADE_RESULTS: usize = 3;
    pub const SIMPLE_SELECT_PLAYERS: usize = 4;
    pub const TIME_TRIAL_SELECT_PLAYERS: usize = 5;
    pub const SAVE_SETUP: usize = 6;
    pub const LOAD_CHAMPIONSHIP_46E899: usize = 7;
    pub const LOAD_CHAMPIONSHIP: usize = 8;
    pub const SAVE_CHAMPIONSHIP: usize = 9;
    pub const LOAD_REPLAY: usize = 10;
    pub const SAVE_REPLAY: usize = 11;
    pub const ARCADE_SELECT_LEVEL: usize = 13;
    pub const MAIN_MENU: usize = 15;
    pub const NETWORK_RALLY: usize = 16;
    pub const OPTIONS: usize = 17;
    pub const PLAYER_DETAILS: usize = 18;
    pub const PLAYER1_DETAILS: usize = 19;
    pub const PLAYER2_DETAILS: usize = 20;
    pub const PLAYER3_DETAILS: usize = 21;
    pub const PLAYER4_DETAILS: usize = 22;
    pub const INFORMATION: usize = 23;
    pub const START_RALLY: usize = 25;
    pub const AUDIO_SETUP: usize = 26;
    pub const ADVANCED_AUDIO: usize = 27;
    pub const LEAVE_QUESTION: usize = 28;
    pub const CAR_SPECIFICATION: usize = 29;
    pub const SLIDESHOW: usize = 30;
    pub const SELECT_CAR: usize = 31;
    pub const CAR_SPECS_47AF4C: usize = 32;
    pub const VIEW_COURSES: usize = 34;
    pub const TIME_TRIAL_STAGE_DETAILS: usize = 35;
    pub const VIEW_BEST_TIMES: usize = 37;
    pub const VIEW_BEST_SCORES: usize = 38;
    pub const BEST_TIMES: usize = 39;
    pub const ARCADE_BEST_TIMES: usize = 40;
    pub const TIME_TRIAL_BEST_TIMES: usize = 41;
    pub const BEST_SCORES_48A69E: usize = 42;
    pub const BEST_CLASS_SCORES: usize = 43;
    pub const BEST_OVERALL_SCORES: usize = 44;
    pub const VISUAL_PREFERENCES: usize = 46;
    pub const VISUAL_OPTIONS: usize = 47;
    pub const PLAYER1_CONTROLLER: usize = 50;
    pub const PLAYER1_BUTTONS: usize = 51;
    pub const PLAYER2_CONTROLLER: usize = 52;
    pub const PLAYER2_BUTTONS: usize = 53;
    pub const PLAYER3_CONTROLLER: usize = 54;
    pub const PLAYER3_BUTTONS: usize = 55;
    pub const PLAYER4_CONTROLLER: usize = 56;
    pub const PLAYER4_BUTTONS: usize = 57;
    pub const REDEFINE_KEYS_PLAYER1: usize = 58;
    pub const REDEFINE_KEYS_PLAYER2: usize = 59;
    pub const REDEFINE_KEYS_PLAYER3: usize = 60;
    pub const REDEFINE_KEYS_PLAYER4: usize = 61;
    pub const CHAMPIONSHIP_NEW_SIMULATION: usize = 64;
    pub const NEW_SIMULATION: usize = 65;
    pub const NEXT_STAGE: usize = 66;
    pub const VIEW_STAGE: usize = 67;
    pub const RALLY_STAGES: usize = 68;
    pub const SERVICE_AREA: usize = 71;
    pub const CLASS_STAGE_RESULTS: usize = 72;
    pub const OVERALL_STAGE_RESULTS: usize = 73;
    pub const CLASS_RALLY_RESULTS: usize = 74;
    pub const OVERALL_RALLY_RESULTS: usize = 75;
    pub const REVIEW_OPTIONS_47D5A5: usize = 79;
    pub const REVIEW_OPTIONS_480D46: usize = 80;
    pub const REVIEW_OPTIONS_48108E: usize = 81;
    pub const REVIEW_OPTIONS_481429: usize = 82;
    pub const INSPECTION_AREA: usize = 83;
    pub const IN_GAME_OPTIONS: usize = 84;
    pub const WELCOME: usize = 85;
    pub const CLASS_CHAMPIONSHIP_STANDINGS: usize = 86;
    pub const CHAMPIONSHIP_STANDINGS: usize = 87;
    pub const SCREEN_488F4C: usize = 88;
    pub const ARCADE_REVIEW_OPTIONS: usize = 89;
    pub const REVIEW_OPTIONS_4882E2: usize = 90;
    pub const REVIEW_OPTIONS_48861D: usize = 91;
    pub const REVIEW_OPTIONS_485F5E: usize = 93;
    pub const STAGE_RESULT: usize = 94;
    pub const RESULTS_48584B: usize = 96;
    pub const RESULTS_482E9B: usize = 98;
    pub const TIME_TRIAL_RESULTS: usize = 99;
    pub const SCREEN_483163: usize = 100;
}

/// The number of slots.
pub const SLOTS: usize = 103;

#[repr(C)]
pub struct ScreenItems {
    /// `dword_747660` ..: the item highlighted when the screen was last left.
    pub current: [Dword; SLOTS],
    /// `dword_7477FC` ..: the item a screen starts on.
    pub first: [Dword; SLOTS],
}

pub static SCREEN_ITEMS: ScreenItems = ScreenItems {
    current: [
        Dword::new(0x2),
        Dword::new(0x2),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x2),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x2),
        Dword::new(0x2),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x4),
        Dword::new(0x2),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x16),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x3),
        Dword::new(0x1),
    ],
    first: [
        Dword::new(0x2),
        Dword::new(0x2),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0xb),
        Dword::new(0xb),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x2),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x2),
        Dword::new(0x2),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x4),
        Dword::new(0x2),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x16),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x1),
        Dword::new(0x3),
        Dword::new(0x1),
    ],
};

/// IDA 0x747660, the first byte of the piece.
pub const START: usize = 0x747660;
/// IDA 0x747998, the end of the piece.
pub const END: usize = 0x747998;

/// Where IDA 0x747660 is now (`moved::MOVED`'s home).
pub fn screen_items_addr() -> u32 {
    crate::pointers::game_address(&raw const SCREEN_ITEMS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn screen_items_start_as_the_games() {
        assert_eq!(size_of::<ScreenItems>(), END - START);
        assert_eq!(offset_of!(ScreenItems, first), 0x7477FC - START);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(screen_items_addr()).cast_const(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
    }
}
