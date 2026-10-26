//! `race::single_rally_race` (0x4843EC, `single_rally`'s race, IDA 2026-10-17): the chosen
//! track looked up (`g_trakPakHeader`), the settings and the players' records
//! (`g_players`) copied into the front end's race variables, the race run
//! (`race::race_from_menu`, with the sound objects and the screen modes round it), and the
//! results copied back into the players' records.
//!
//! Plain Rust, written from the original's 902 instructions: copies between a record and
//! the game's variables (lists of (offset, variable) in the original's order), compares and
//! jumps (now `if`s and one `while`), the calls (`step`, by place: the index of the
//! original's call in `table::CALLS` plus one), `rep movsb` (`ptr::copy`), two divisions by
//! 10 and one `fld` / `fistp`. The original's registers carry no value from one part to
//! the next (each `pusha` .. `popa` restores them, and what the calls leave in eax, ebx,
//! ecx, edx and al is overwritten before it is read), so there are none here.
//!
//! The proof is the test at the end of the file: the original's instruction table (`table`,
//! `cfg(test)` only, with its small interpreter) and this code run on the same synthetic
//! game memory and players' records, with calls that record themselves instead of running;
//! the memory they leave, the calls (with the memory at each of them) and the result are
//! compared.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{game_addr, rva};
use crate::menu::arcade_store::ARCADE_GAME;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::players_store::PLAYER_TABLE;
use crate::pointers::GamePtr;
use crate::race::state_store::RACE_STATE;
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::session::{Bytes, Dword, Float};

/// A player's or a track's record: a game address.
type Record = u32;

// g_trakPakHeader (0x734BC6, `records::stages_store`): +0x34 the entries, 0x40 apart. An
// entry (at `edi`) holds its track number at +0x34; the list ends at a dword -1 where the
// next entry would start.
/// g_players (0x75647C): the players' records, 4 bytes each.
const PLAYERS: &Dword = &PLAYER_TABLE.pointers[0];
/// dword_7556F4: the standings summary, copied to `RACE_INFO`.
const SUMMARY: &Dword = &CHAMP_VIEW.summary;
/// dword_7558DC, copied to dword_5C2FD0 after the loading picture.
const AFTER_LOADING: &Dword = &EVENT_STATE.sparks_mode;
/// dword_7554C8: kept over the race (`RACE_KEPT`).
const KEPT: &Dword = &SCREEN_FLOW.kept;
/// dword_755984: the number of players.
const PLAYER_COUNT: &Dword = &EVENT_STATE.player_count;
/// dword_755988: the event mode (5, 4, 0xF, 0xA: the events' one-car races).
const EVENT_MODE: &Dword = &EVENT_STATE.event_kind;
/// dword_75598C: the current player's number.
const CURRENT: &Dword = &EVENT_STATE.current_player;
/// dword_7587E8: the run's start argument (1: no loading picture after).
const SWITCH: &Dword = &ARCADE_GAME.scene_let_go;
/// dword_75ACD0: a copy of `SUMMARY`.
const RACE_INFO: &Dword = &RACE_STATE.settings.rivals;
/// dword_75ACD4: the stage chosen.
const STAGE: &Dword = &RACE_STATE.settings.stage;
/// dword_75ACD8: the run's kind argument.
const RACE_KIND: &Dword = &RACE_STATE.settings.run_kind;
/// dword_75ACDC: 1 when the race is not an event's and has more than one player.
const NO_REPLAY: &Dword = &RACE_STATE.settings.no_replay;
/// dword_75ACE0: 1, or 2 in the events' one-car modes.
const RACE_MODE: &Dword = &RACE_STATE.settings.race_mode;
/// dword_75ACE4: a copy of `KEPT`, put back after the race.
const RACE_KEPT: &Dword = &RACE_STATE.settings.kept;
/// dword_75B028 / dword_75B098: the current / the first car's result (-1 none, -2 left).
const RESULT: &Dword = &RACE_STATE.results.results[0].result;
const RESULT_2: &Dword = &RACE_STATE.results.results[1].result;
/// dword_75ACE8 / dword_75ACEC: the record's word at +8, of the current player / of player 1.
const WORD_8: &Dword = &RACE_STATE.settings.car_kind_current;
const WORD_8_FIRST: &Dword = &RACE_STATE.settings.car_kind_first;
/// dword_75AE24: the second set's word that mode 2 takes from +0x10.
const MODE_WORD: &Dword = &RACE_STATE.settings.cars[0].view;
/// The two blocks of 0x20 bytes copied from a record's +0x24C, the first car's / the current's.
const BLOCK_FIRST: &Dword = &RACE_STATE.results.results[1].block[0];
const BLOCK_CURRENT: &Dword = &RACE_STATE.results.results[0].block[0];
/// The first car's five times (added, then divided by 10 into the record's +0xD8) and the
/// word whose six times is added; the current car's.
const TIMES_FIRST: [&Dword; 4] = [
    &RACE_STATE.settings.cars[1].wear[10],
    &RACE_STATE.settings.cars[1].wear[11],
    &RACE_STATE.settings.cars[1].wear[12],
    &RACE_STATE.settings.cars[1].wear[13],
];
const EXTRA_FIRST: &Dword = &RACE_STATE.settings.cars[1].wear[14];
const TIMES_CURRENT: [&Dword; 4] = [
    &RACE_STATE.settings.cars[0].wear[10],
    &RACE_STATE.settings.cars[0].wear[11],
    &RACE_STATE.settings.cars[0].wear[12],
    &RACE_STATE.settings.cars[0].wear[13],
];
const EXTRA_CURRENT: &Dword = &RACE_STATE.settings.cars[0].wear[14];
/// dword_75B00C: the current car's word that goes back to its record's +0x284.
const BACK_284_CURRENT: &Dword = &RACE_STATE.results.hud_bits_current;

/// The fatal error of a track number not in the list.
const UNKNOWN_TRACK: FatalError = FatalError {
    code: 0x8073_9281,
    message: "Unknown Shaun TrakNum",
};

/// The car blocks a race leaves its result in: the result word, the time, one more word, the
/// seconds (a float), and the 0x40 bytes of data.
struct Outcome {
    result: &'static Dword,
    time: &'static Dword,
    word: &'static Dword,
    secs: &'static Float,
    data: &'static Bytes<0x40>,
}
const OUTCOME_FIRST: Outcome = Outcome {
    result: &RACE_STATE.results.results[1].result,
    time: &RACE_STATE.results.results[1].time,
    word: &RACE_STATE.results.results[1].word,
    secs: &RACE_STATE.results.results[1].secs,
    data: &RACE_STATE.results.results[1].data,
};
const OUTCOME_CURRENT: Outcome = Outcome {
    result: &RACE_STATE.results.results[0].result,
    time: &RACE_STATE.results.results[0].time,
    word: &RACE_STATE.results.results[0].word,
    secs: &RACE_STATE.results.results[0].secs,
    data: &RACE_STATE.results.results[0].data,
};

/// The track entry's two fields copied into the race variables (record offset, variable).
const TRACK_TO_RACE: [(u32, &Dword); 2] = [
    (0x1C, &RACE_STATE.settings.track_1c),
    (0x20, &RACE_STATE.settings.track_20),
];

/// Five settings words copied to the race variables (from, to).
const SETTINGS_TO_RACE: [(&Dword, &Dword); 5] = [
    (&OPTION_WORK.setting_0, &RACE_STATE.settings.setting_0),
    (&OPTION_WORK.setting_1, &RACE_STATE.settings.setting_1),
    (&OPTION_WORK.setting_2, &RACE_STATE.settings.setting_2),
    (&OPTION_WORK.setting_3, &RACE_STATE.settings.setting_3),
    (&OPTION_WORK.setting_4, &RACE_STATE.settings.setting_4),
];

/// Player 1's record, words (offset, variable), into the race's common variables.
const PLAYER_1_WORDS: [(u32, &Dword); 19] = [
    (0x2C, &RACE_STATE.settings.cars[1].record_2c),
    (0x30, &RACE_STATE.settings.cars[1].record_30),
    (0x34, &RACE_STATE.settings.cars[1].record_34[0]),
    (0x38, &RACE_STATE.settings.cars[1].record_34[1]),
    (0x3C, &RACE_STATE.settings.cars[1].record_34[2]),
    (0x40, &RACE_STATE.settings.cars[1].record_34[3]),
    (0x44, &RACE_STATE.settings.cars[1].record_34[4]),
    (0x48, &RACE_STATE.settings.cars[1].record_34[5]),
    (0x50, &RACE_STATE.settings.cars[1].record_50[0]),
    (0x54, &RACE_STATE.settings.cars[1].record_50[1]),
    (0x58, &RACE_STATE.settings.cars[1].record_50[2]),
    (0x5C, &RACE_STATE.settings.cars[1].record_50[3]),
    (0x60, &RACE_STATE.settings.cars[1].record_50[4]),
    (0x64, &RACE_STATE.settings.cars[1].record_50[5]),
    (0x68, &RACE_STATE.settings.cars[1].record_50[6]),
    (0x6C, &RACE_STATE.settings.cars[1].record_50[7]),
    (0x70, &RACE_STATE.settings.cars[1].record_50[8]),
    (0x74, &RACE_STATE.settings.cars[1].record_74),
    (0x78, &RACE_STATE.settings.cars[1].record_78),
];

/// Player 1's record into the first car's variables, before the 0x20 bytes at +0x24C.
const FIRST_HEAD: [(u32, &Dword); 8] = [
    (0x8, &RACE_STATE.settings.car_kind_first),
    (0x10C, &RACE_STATE.settings.cars[1].tyre),
    (0x110, &RACE_STATE.settings.cars[1].brake_balance),
    (0x114, &RACE_STATE.settings.cars[1].ride_height),
    (0x118, &RACE_STATE.settings.cars[1].suspension_stiffness),
    (0x11C, &RACE_STATE.settings.cars[1].steering_sensitivity),
    (0x120, &RACE_STATE.settings.cars[1].gear_ratio),
    (0x288, &RACE_STATE.results.word_288_first),
];

/// Player 1's record into the first car's variables, after them.
const FIRST_TAIL: [(u32, &Dword); 65] = [
    (0x278, &RACE_STATE.results.flags_first),
    (0xD4, &RACE_STATE.settings.cars[1].damage[0]),
    (0xD8, &RACE_STATE.settings.cars[1].damage[1]),
    (0xDC, &RACE_STATE.settings.cars[1].damage[2]),
    (0xE0, &RACE_STATE.settings.cars[1].damage[3]),
    (0xE4, &RACE_STATE.settings.cars[1].damage[4]),
    (0xE8, &RACE_STATE.settings.cars[1].damage[5]),
    (0xEC, &RACE_STATE.settings.cars[1].damage[6]),
    (0xF0, &RACE_STATE.settings.cars[1].damage[7]),
    (0xF4, &RACE_STATE.settings.cars[1].damage[8]),
    (0xF8, &RACE_STATE.settings.cars[1].damage[9]),
    (0xFC, &RACE_STATE.settings.cars[1].damage[10]),
    (0x100, &RACE_STATE.settings.cars[1].damage[11]),
    (0x104, &RACE_STATE.settings.cars[1].wear[9]),
    (0x108, &RACE_STATE.settings.cars[1].damage[12]),
    (0x158, &RACE_STATE.settings.cars[1].map_area),
    (0x15C, &RACE_STATE.settings.cars[1].rear_mirror),
    (0x160, &RACE_STATE.settings.cars[1].rev_counter),
    (0x168, &RACE_STATE.settings.cars[1].message_display),
    (0x164, &RACE_STATE.settings.cars[1].car_info),
    (0x170, &RACE_STATE.settings.cars[1].driver_info),
    (0x16C, &RACE_STATE.settings.cars[1].progress_bar),
    (0x174, &RACE_STATE.settings.cars[1].pace_notes),
    (0x178, &RACE_STATE.settings.cars[1].metric_units),
    (0x284, &RACE_STATE.results.hud_bits_first),
    (0x17C, &RACE_STATE.settings.cars[1].wear[0]),
    (0x180, &RACE_STATE.settings.cars[1].wear[1]),
    (0x184, &RACE_STATE.settings.cars[1].wear[2]),
    (0x188, &RACE_STATE.settings.cars[1].wear[3]),
    (0x18C, &RACE_STATE.settings.cars[1].wear[4]),
    (0x190, &RACE_STATE.settings.cars[1].wear[5]),
    (0x194, &RACE_STATE.settings.cars[1].wear[6]),
    (0x198, &RACE_STATE.settings.cars[1].wear[7]),
    (0x19C, &RACE_STATE.settings.cars[1].wear[8]),
    (0x1A0, &RACE_STATE.settings.cars[1].wear[9]),
    (0x1A4, &RACE_STATE.settings.cars[1].wear[10]),
    (0x1A8, &RACE_STATE.settings.cars[1].wear[11]),
    (0x1AC, &RACE_STATE.settings.cars[1].wear[12]),
    (0x1B0, &RACE_STATE.settings.cars[1].wear[13]),
    (0x1B4, &RACE_STATE.settings.cars[1].wear[14]),
    (0x1B8, &RACE_STATE.settings.cars[1].wear[15]),
    (0x1BC, &RACE_STATE.settings.cars[1].wear[16]),
    (0x1C0, &RACE_STATE.settings.cars[1].wear[17]),
    (0x1C4, &RACE_STATE.settings.cars[1].wear[18]),
    (0x1C8, &RACE_STATE.settings.cars[1].wear[19]),
    (0x1CC, &RACE_STATE.settings.cars[1].wear[20]),
    (0x1D0, &RACE_STATE.settings.cars[1].wear[21]),
    (0x1D4, &RACE_STATE.settings.cars[1].wear[22]),
    (0x1D8, &RACE_STATE.settings.cars[1].wear[23]),
    (0x1DC, &RACE_STATE.settings.cars[1].wear[24]),
    (0x1E0, &RACE_STATE.settings.cars[1].wear[25]),
    (0x1E4, &RACE_STATE.settings.cars[1].wear[26]),
    (0x1E8, &RACE_STATE.settings.cars[1].wear[27]),
    (0x1EC, &RACE_STATE.settings.cars[1].wear[28]),
    (0x1F0, &RACE_STATE.settings.cars[1].wear[29]),
    (0x1F4, &RACE_STATE.settings.cars[1].wear[30]),
    (0x1F8, &RACE_STATE.settings.cars[1].wear[31]),
    (0x1FC, &RACE_STATE.settings.cars[1].wear[32]),
    (0x200, &RACE_STATE.settings.cars[1].wear[33]),
    (0x204, &RACE_STATE.settings.cars[1].wear[34]),
    (0x208, &RACE_STATE.settings.cars[1].wear[35]),
    (0x28, &RACE_STATE.settings.cars[1].race_number),
    (0x154, &RACE_STATE.settings.cars[1].codriver_sex),
    (0xC, &RACE_STATE.settings.cars[1].view),
    (0x10, &RACE_STATE.settings.cars[1].view),
];

/// The current player's record into the second set of variables, before the 0x20 bytes at +0x24C.
const SECOND_HEAD: [(u32, &Dword); 8] = [
    (0x8, &RACE_STATE.settings.car_kind_current),
    (0x10C, &RACE_STATE.settings.cars[0].tyre),
    (0x110, &RACE_STATE.settings.cars[0].brake_balance),
    (0x114, &RACE_STATE.settings.cars[0].ride_height),
    (0x118, &RACE_STATE.settings.cars[0].suspension_stiffness),
    (0x11C, &RACE_STATE.settings.cars[0].steering_sensitivity),
    (0x120, &RACE_STATE.settings.cars[0].gear_ratio),
    (0x288, &RACE_STATE.results.word_288_current),
];

/// The current player's record, after them.
const SECOND_MID: [(u32, &Dword); 64] = [
    (0x278, &RACE_STATE.results.flags_current),
    (0xD4, &RACE_STATE.settings.cars[0].damage[0]),
    (0xD8, &RACE_STATE.settings.cars[0].damage[1]),
    (0xDC, &RACE_STATE.settings.cars[0].damage[2]),
    (0xE0, &RACE_STATE.settings.cars[0].damage[3]),
    (0xE4, &RACE_STATE.settings.cars[0].damage[4]),
    (0xE8, &RACE_STATE.settings.cars[0].damage[5]),
    (0xEC, &RACE_STATE.settings.cars[0].damage[6]),
    (0xF0, &RACE_STATE.settings.cars[0].damage[7]),
    (0xF4, &RACE_STATE.settings.cars[0].damage[8]),
    (0xF8, &RACE_STATE.settings.cars[0].damage[9]),
    (0xFC, &RACE_STATE.settings.cars[0].damage[10]),
    (0x100, &RACE_STATE.settings.cars[0].damage[11]),
    (0x104, &RACE_STATE.settings.cars[0].wear[9]),
    (0x108, &RACE_STATE.settings.cars[0].damage[12]),
    (0x158, &RACE_STATE.settings.cars[0].map_area),
    (0x15C, &RACE_STATE.settings.cars[0].rear_mirror),
    (0x160, &RACE_STATE.settings.cars[0].rev_counter),
    (0x168, &RACE_STATE.settings.cars[0].message_display),
    (0x164, &RACE_STATE.settings.cars[0].car_info),
    (0x170, &RACE_STATE.settings.cars[0].driver_info),
    (0x16C, &RACE_STATE.settings.cars[0].progress_bar),
    (0x174, &RACE_STATE.settings.cars[0].pace_notes),
    (0x178, &RACE_STATE.settings.cars[0].metric_units),
    (0x284, &RACE_STATE.results.hud_bits_current),
    (0x17C, &RACE_STATE.settings.cars[0].wear[0]),
    (0x180, &RACE_STATE.settings.cars[0].wear[1]),
    (0x184, &RACE_STATE.settings.cars[0].wear[2]),
    (0x188, &RACE_STATE.settings.cars[0].wear[3]),
    (0x18C, &RACE_STATE.settings.cars[0].wear[4]),
    (0x190, &RACE_STATE.settings.cars[0].wear[5]),
    (0x194, &RACE_STATE.settings.cars[0].wear[6]),
    (0x198, &RACE_STATE.settings.cars[0].wear[7]),
    (0x19C, &RACE_STATE.settings.cars[0].wear[8]),
    (0x1A0, &RACE_STATE.settings.cars[0].wear[9]),
    (0x1A4, &RACE_STATE.settings.cars[0].wear[10]),
    (0x1A8, &RACE_STATE.settings.cars[0].wear[11]),
    (0x1AC, &RACE_STATE.settings.cars[0].wear[12]),
    (0x1B0, &RACE_STATE.settings.cars[0].wear[13]),
    (0x1B4, &RACE_STATE.settings.cars[0].wear[14]),
    (0x1B8, &RACE_STATE.settings.cars[0].wear[15]),
    (0x1BC, &RACE_STATE.settings.cars[0].wear[16]),
    (0x1C0, &RACE_STATE.settings.cars[0].wear[17]),
    (0x1C4, &RACE_STATE.settings.cars[0].wear[18]),
    (0x1C8, &RACE_STATE.settings.cars[0].wear[19]),
    (0x1CC, &RACE_STATE.settings.cars[0].wear[20]),
    (0x1D0, &RACE_STATE.settings.cars[0].wear[21]),
    (0x1D4, &RACE_STATE.settings.cars[0].wear[22]),
    (0x1D8, &RACE_STATE.settings.cars[0].wear[23]),
    (0x1DC, &RACE_STATE.settings.cars[0].wear[24]),
    (0x1E0, &RACE_STATE.settings.cars[0].wear[25]),
    (0x1E4, &RACE_STATE.settings.cars[0].wear[26]),
    (0x1E8, &RACE_STATE.settings.cars[0].wear[27]),
    (0x1EC, &RACE_STATE.settings.cars[0].wear[28]),
    (0x1F0, &RACE_STATE.settings.cars[0].wear[29]),
    (0x1F4, &RACE_STATE.settings.cars[0].wear[30]),
    (0x1F8, &RACE_STATE.settings.cars[0].wear[31]),
    (0x1FC, &RACE_STATE.settings.cars[0].wear[32]),
    (0x200, &RACE_STATE.settings.cars[0].wear[33]),
    (0x204, &RACE_STATE.settings.cars[0].wear[34]),
    (0x208, &RACE_STATE.settings.cars[0].wear[35]),
    (0x28, &RACE_STATE.settings.cars[0].race_number),
    (0x154, &RACE_STATE.settings.cars[0].codriver_sex),
    (0xC, &RACE_STATE.settings.cars[0].view),
];

/// The current player's record, after the mode-2 word.
const SECOND_TAIL: [(u32, &Dword); 19] = [
    (0x2C, &RACE_STATE.settings.cars[0].record_2c),
    (0x30, &RACE_STATE.settings.cars[0].record_30),
    (0x34, &RACE_STATE.settings.cars[0].record_34[0]),
    (0x38, &RACE_STATE.settings.cars[0].record_34[1]),
    (0x3C, &RACE_STATE.settings.cars[0].record_34[2]),
    (0x40, &RACE_STATE.settings.cars[0].record_34[3]),
    (0x44, &RACE_STATE.settings.cars[0].record_34[4]),
    (0x48, &RACE_STATE.settings.cars[0].record_34[5]),
    (0x50, &RACE_STATE.settings.cars[0].record_50[0]),
    (0x54, &RACE_STATE.settings.cars[0].record_50[1]),
    (0x58, &RACE_STATE.settings.cars[0].record_50[2]),
    (0x5C, &RACE_STATE.settings.cars[0].record_50[3]),
    (0x60, &RACE_STATE.settings.cars[0].record_50[4]),
    (0x64, &RACE_STATE.settings.cars[0].record_50[5]),
    (0x68, &RACE_STATE.settings.cars[0].record_50[6]),
    (0x6C, &RACE_STATE.settings.cars[0].record_50[7]),
    (0x70, &RACE_STATE.settings.cars[0].record_50[8]),
    (0x74, &RACE_STATE.settings.cars[0].record_74),
    (0x78, &RACE_STATE.settings.cars[0].record_78),
];

/// The first set of variables back into player 1's record (variable, offset), up to the total.
const FIRST_BACK_A: [(&Dword, u32); 52] = [
    (&RACE_STATE.results.hud_bits_first, 0x284),
    (&RACE_STATE.results.flags_first, 0x278),
    (&RACE_STATE.settings.cars[1].damage[0], 0xD4),
    (&RACE_STATE.settings.cars[1].damage[1], 0xD8),
    (&RACE_STATE.settings.cars[1].damage[2], 0xDC),
    (&RACE_STATE.settings.cars[1].damage[3], 0xE0),
    (&RACE_STATE.settings.cars[1].damage[4], 0xE4),
    (&RACE_STATE.settings.cars[1].damage[5], 0xE8),
    (&RACE_STATE.settings.cars[1].damage[6], 0xEC),
    (&RACE_STATE.settings.cars[1].damage[7], 0xF0),
    (&RACE_STATE.settings.cars[1].damage[8], 0xF4),
    (&RACE_STATE.settings.cars[1].damage[9], 0xF8),
    (&RACE_STATE.settings.cars[1].damage[10], 0xFC),
    (&RACE_STATE.settings.cars[1].damage[11], 0x100),
    (&RACE_STATE.settings.cars[1].wear[9], 0x104),
    (&RACE_STATE.settings.cars[1].damage[12], 0x108),
    (&RACE_STATE.settings.cars[1].wear[0], 0x17C),
    (&RACE_STATE.settings.cars[1].wear[1], 0x180),
    (&RACE_STATE.settings.cars[1].wear[2], 0x184),
    (&RACE_STATE.settings.cars[1].wear[3], 0x188),
    (&RACE_STATE.settings.cars[1].wear[4], 0x18C),
    (&RACE_STATE.settings.cars[1].wear[5], 0x190),
    (&RACE_STATE.settings.cars[1].wear[6], 0x194),
    (&RACE_STATE.settings.cars[1].wear[7], 0x198),
    (&RACE_STATE.settings.cars[1].wear[8], 0x19C),
    (&RACE_STATE.settings.cars[1].wear[9], 0x1A0),
    (&RACE_STATE.settings.cars[1].wear[10], 0x1A4),
    (&RACE_STATE.settings.cars[1].wear[11], 0x1A8),
    (&RACE_STATE.settings.cars[1].wear[12], 0x1AC),
    (&RACE_STATE.settings.cars[1].wear[13], 0x1B0),
    (&RACE_STATE.settings.cars[1].wear[14], 0x1B4),
    (&RACE_STATE.settings.cars[1].wear[15], 0x1B8),
    (&RACE_STATE.settings.cars[1].wear[16], 0x1BC),
    (&RACE_STATE.settings.cars[1].wear[17], 0x1C0),
    (&RACE_STATE.settings.cars[1].wear[18], 0x1C4),
    (&RACE_STATE.settings.cars[1].wear[19], 0x1C8),
    (&RACE_STATE.settings.cars[1].wear[20], 0x1CC),
    (&RACE_STATE.settings.cars[1].wear[21], 0x1D0),
    (&RACE_STATE.settings.cars[1].wear[22], 0x1D4),
    (&RACE_STATE.settings.cars[1].wear[23], 0x1D8),
    (&RACE_STATE.settings.cars[1].wear[24], 0x1DC),
    (&RACE_STATE.settings.cars[1].wear[25], 0x1E0),
    (&RACE_STATE.settings.cars[1].wear[26], 0x1E4),
    (&RACE_STATE.settings.cars[1].wear[27], 0x1E8),
    (&RACE_STATE.settings.cars[1].wear[28], 0x1EC),
    (&RACE_STATE.settings.cars[1].wear[29], 0x1F0),
    (&RACE_STATE.settings.cars[1].wear[30], 0x1F4),
    (&RACE_STATE.settings.cars[1].wear[31], 0x1F8),
    (&RACE_STATE.settings.cars[1].wear[32], 0x1FC),
    (&RACE_STATE.settings.cars[1].wear[33], 0x200),
    (&RACE_STATE.settings.cars[1].wear[34], 0x204),
    (&RACE_STATE.settings.cars[1].wear[35], 0x208),
];

/// The same, after the total.
const FIRST_BACK_B: [(&Dword, u32); 6] = [
    (&RACE_STATE.settings.cars[1].wear[27], 0xF8),
    (&RACE_STATE.settings.cars[1].wear[35], 0x108),
    (&RACE_STATE.settings.cars[1].wear[30], 0x100),
    (&RACE_STATE.settings.cars[1].wear[22], 0xE8),
    (&RACE_STATE.settings.cars[1].codriver_sex, 0x154),
    (&RACE_STATE.settings.cars[1].view, 0xC),
];

/// The second set back into the current player's record (variable, offset), up to the total.
const SECOND_BACK_A: [(&Dword, u32); 50] = [
    (&RACE_STATE.results.flags_current, 0x278),
    (&RACE_STATE.settings.cars[0].damage[0], 0xD4),
    (&RACE_STATE.settings.cars[0].damage[1], 0xD8),
    (&RACE_STATE.settings.cars[0].damage[2], 0xDC),
    (&RACE_STATE.settings.cars[0].damage[3], 0xE0),
    (&RACE_STATE.settings.cars[0].damage[4], 0xE4),
    (&RACE_STATE.settings.cars[0].damage[5], 0xE8),
    (&RACE_STATE.settings.cars[0].damage[6], 0xEC),
    (&RACE_STATE.settings.cars[0].damage[7], 0xF0),
    (&RACE_STATE.settings.cars[0].damage[8], 0xF4),
    (&RACE_STATE.settings.cars[0].damage[10], 0xFC),
    (&RACE_STATE.settings.cars[0].damage[11], 0x100),
    (&RACE_STATE.settings.cars[0].wear[9], 0x104),
    (&RACE_STATE.settings.cars[0].damage[12], 0x108),
    (&RACE_STATE.settings.cars[0].wear[0], 0x17C),
    (&RACE_STATE.settings.cars[0].wear[1], 0x180),
    (&RACE_STATE.settings.cars[0].wear[2], 0x184),
    (&RACE_STATE.settings.cars[0].wear[3], 0x188),
    (&RACE_STATE.settings.cars[0].wear[4], 0x18C),
    (&RACE_STATE.settings.cars[0].wear[5], 0x190),
    (&RACE_STATE.settings.cars[0].wear[6], 0x194),
    (&RACE_STATE.settings.cars[0].wear[7], 0x198),
    (&RACE_STATE.settings.cars[0].wear[8], 0x19C),
    (&RACE_STATE.settings.cars[0].wear[9], 0x1A0),
    (&RACE_STATE.settings.cars[0].wear[10], 0x1A4),
    (&RACE_STATE.settings.cars[0].wear[11], 0x1A8),
    (&RACE_STATE.settings.cars[0].wear[12], 0x1AC),
    (&RACE_STATE.settings.cars[0].wear[13], 0x1B0),
    (&RACE_STATE.settings.cars[0].wear[14], 0x1B4),
    (&RACE_STATE.settings.cars[0].wear[15], 0x1B8),
    (&RACE_STATE.settings.cars[0].wear[16], 0x1BC),
    (&RACE_STATE.settings.cars[0].wear[17], 0x1C0),
    (&RACE_STATE.settings.cars[0].wear[18], 0x1C4),
    (&RACE_STATE.settings.cars[0].wear[19], 0x1C8),
    (&RACE_STATE.settings.cars[0].wear[20], 0x1CC),
    (&RACE_STATE.settings.cars[0].wear[21], 0x1D0),
    (&RACE_STATE.settings.cars[0].wear[22], 0x1D4),
    (&RACE_STATE.settings.cars[0].wear[23], 0x1D8),
    (&RACE_STATE.settings.cars[0].wear[24], 0x1DC),
    (&RACE_STATE.settings.cars[0].wear[25], 0x1E0),
    (&RACE_STATE.settings.cars[0].wear[26], 0x1E4),
    (&RACE_STATE.settings.cars[0].wear[27], 0x1E8),
    (&RACE_STATE.settings.cars[0].wear[28], 0x1EC),
    (&RACE_STATE.settings.cars[0].wear[29], 0x1F0),
    (&RACE_STATE.settings.cars[0].wear[30], 0x1F4),
    (&RACE_STATE.settings.cars[0].wear[31], 0x1F8),
    (&RACE_STATE.settings.cars[0].wear[32], 0x1FC),
    (&RACE_STATE.settings.cars[0].wear[33], 0x200),
    (&RACE_STATE.settings.cars[0].wear[34], 0x204),
    (&RACE_STATE.settings.cars[0].wear[35], 0x208),
];

/// The same, after the total.
const SECOND_BACK_B: [(&Dword, u32); 6] = [
    (&RACE_STATE.settings.cars[0].wear[27], 0xF8),
    (&RACE_STATE.settings.cars[0].wear[35], 0x108),
    (&RACE_STATE.settings.cars[0].wear[30], 0x100),
    (&RACE_STATE.settings.cars[0].wear[22], 0xE8),
    (&RACE_STATE.settings.cars[0].codriver_sex, 0x154),
    (&RACE_STATE.settings.cars[0].view, 0xC),
];

fn mem(a: usize) -> *mut u32 {
    crate::pointers::host(game_addr(rva(a))) as *mut u32
}

/// A variable of the game that the run's steps read and write: a cell, or an address of the
/// image's.
trait Var {
    unsafe fn load(self) -> u32;
    unsafe fn store(self, v: u32);
}

impl Var for usize {
    unsafe fn load(self) -> u32 {
        unsafe { mem(self).read_unaligned() }
    }

    unsafe fn store(self, v: u32) {
        unsafe { mem(self).write_unaligned(v) }
    }
}

impl Var for &Dword {
    unsafe fn load(self) -> u32 {
        self.get()
    }

    unsafe fn store(self, v: u32) {
        self.set(v)
    }
}

unsafe fn var(a: impl Var) -> u32 {
    unsafe { a.load() }
}

unsafe fn set_var(a: impl Var, v: u32) {
    unsafe { a.store(v) }
}

fn field(rec: Record, off: u32) -> Result<u32, BadInput> {
    GamePtr::new(rec.wrapping_add(off)).read("a player's record")
}

fn set_field(rec: Record, off: u32, v: u32) -> Result<(), BadInput> {
    GamePtr::new(rec.wrapping_add(off)).write(v, "a player's record")
}

/// `g_players`[n].
pub fn player(n: u32) -> Result<u32, BadInput> {
    GamePtr::new((PLAYERS.addr()).wrapping_add(n.wrapping_mul(4))).read("the players")
}

/// Words of a record into game variables, in order.
fn take_words(rec: Record, list: &[(u32, &Dword)]) -> Result<(), BadInput> {
    for &(off, to) in list {
        to.set(field(rec, off)?);
    }
    Ok(())
}

/// The 12 bytes at a record's +0x1C .. +0x27 into a car's `keys`.
fn take_keys(rec: Record, keys: &Bytes<12>) -> Result<(), BadInput> {
    let bytes = GamePtr::new(rec.wrapping_add(0x1C)).read_bytes(12, "a player's keys")?;
    for (k, &b) in bytes.iter().enumerate() {
        keys.set(k, b);
    }
    Ok(())
}

/// Game variables into words of a record, in order.
fn give_words(rec: Record, list: &[(&Dword, u32)]) -> Result<(), BadInput> {
    for &(from, off) in list {
        set_field(rec, off, from.get())?;
    }
    Ok(())
}

/// `rep movsb` of `len` bytes.
fn copy_bytes(from: u32, to: u32, len: u32) -> Result<(), BadInput> {
    GamePtr::new(from).copy_to(GamePtr::new(to), len as usize, "a player's record")
}

/// The total a record keeps at +0xD8: a tenth of the sum of four times, and a tenth of six
/// times the extra word. The original divides with `div ebp` (edx 0, ebp 10), after a test
/// of the dividend for 0 that skips the division, so its fatal error 0x80701010 is out of
/// reach.
fn total(times: [&Dword; 4], extra: &Dword) -> u32 {
    let sum = times[0]
        .get()
        .wrapping_add(times[1].get())
        .wrapping_add(times[2].get())
        .wrapping_add(times[3].get());
    (sum / 10).wrapping_add(extra.get().wrapping_mul(6) / 10)
}

/// A car's outcome into its record: a result of -1 or -2 puts the time (+0x88) at 0x36EE80
/// and the state (+0xD0) at 6; any other copies the time, a word (+0x27C), the seconds
/// rounded times 10 (+0x94) and 0x40 bytes of data (+0x20C).
fn give_outcome(rec: Record, o: &Outcome) -> Result<(), BadInput> {
    {
        let r = o.result.get();
        if r == -2i32 as u32 || r == -1i32 as u32 {
            set_field(rec, 0x88, 0x36EE80)?;
            set_field(rec, 0xD0, 6)?;
        } else {
            set_field(rec, 0x88, o.time.get())?;
            set_field(rec, 0x27C, o.word.get())?;
            let secs = crate::math::x87::to_int(o.secs.bits()) as u32;
            set_field(rec, 0x94, secs.wrapping_mul(10))?;
            copy_bytes(o.data.addr(), rec.wrapping_add(0x20C), 0x40)?;
        }
    }
    Ok(())
}

/// `race::single_rally_race` (0x4843EC): `start` (eax) the run's start (1: the loading
/// picture is not shown again after), `kind` (ebx) the kind (3: the rally's times copied
/// first). Refuses, before anything, a missing player record (`g_players`[1] or
/// [`dword_75598C`]; the original reads and writes through it).
pub unsafe fn single_rally_race(start: u32, kind: u32) -> Result<(), Refused> {
    unsafe { guarded(start, kind, &mut step) }
}

/// The refusal, then the race, its calls done by `step`.
unsafe fn guarded(
    start: u32,
    kind: u32,
    step: &mut dyn FnMut(u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        if player(1)? == 0 || player(var(CURRENT))? == 0 {
            return Err(crate::errors::BadInput("a player's record missing").into());
        }
        single_rally(start, kind, step)
    }
}

/// The last calls of the original (from 0x897 of the table): the kept word back, the tune.
unsafe fn finish(step: &mut dyn FnMut(u32) -> Result<(), Refused>) -> Result<(), Refused> {
    unsafe {
        set_var(KEPT, RACE_KEPT.get());
        step(14) // sound::play_tune_3
    }
}

/// The original's body; `step(place)` does the call at `place`.
unsafe fn single_rally(
    start: u32,
    kind: u32,
    step: &mut dyn FnMut(u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        if kind == 3 {
            step(1)?; // race::copy_rally_times
        }
        RACE_INFO.set(var(SUMMARY));
        set_var(SWITCH, start);
        RACE_KIND.set(kind);
        RESULT.set(0);
        RESULT_2.set(0);
        RACE_KEPT.set(0);
        RACE_MODE.set(1);
        step(2)?; // sound::stop_music
        step(3)?; // race::set_race_screen_mode
        step(4)?; // menu::show_loading_picture
        set_var(&VIEWER_PANEL.after_loading_copy, var(AFTER_LOADING));

        // The stage's entry in the track list.
        let stage = STAGE.get();
        let mut entry = crate::records::stages_store::STAGE_TABLES
            .trak_pak()
            .wrapping_add(0x34);
        while field(entry, 0x34)? != stage {
            entry = entry.wrapping_add(0x40);
            if field(entry, 0)? == u32::MAX {
                return Err(UNKNOWN_TRACK.into());
            }
        }
        take_words(entry, &TRACK_TO_RACE)?;
        for &(from, to) in &SETTINGS_TO_RACE {
            to.set(var(from));
        }
        let first = player(1)?;
        take_words(first, &PLAYER_1_WORDS)?;
        take_keys(first, &RACE_STATE.settings.cars[1].keys)?;

        let one_car = matches!(var(EVENT_MODE), 5 | 4 | 0xF | 0xA);
        if one_car {
            RACE_KEPT.set(var(KEPT));
            RACE_MODE.set(2);
            if field(first, 0xD0)? != 1 {
                RESULT_2.set(u32::MAX);
            }
            WORD_8_FIRST.set(field(first, 0x8)?);
            take_words(first, &FIRST_HEAD)?;
            copy_bytes(first.wrapping_add(0x24C), BLOCK_FIRST.addr(), 0x20)?;
            take_words(first, &FIRST_TAIL)?;
            set_var(CURRENT, 0);
        } else if field(player(var(CURRENT))?, 0xD0)? != 1 {
            // The current player's record is not in use: no race.
            RESULT.set(u32::MAX);
            return finish(step);
        }

        // The current player's record (player 0 after an event's one-car race).
        let rec = player(var(CURRENT))?;
        WORD_8.set(field(rec, 0x8)?);
        if field(rec, 0xD0)? != 1 {
            RESULT.set(u32::MAX);
        }
        take_words(rec, &SECOND_HEAD)?;
        copy_bytes(rec.wrapping_add(0x24C), BLOCK_CURRENT.addr(), 0x20)?;
        take_words(rec, &SECOND_MID)?;
        if RACE_MODE.get() == 2 {
            MODE_WORD.set(field(rec, 0x10)?);
        }
        take_words(rec, &SECOND_TAIL)?;
        take_keys(rec, &RACE_STATE.settings.cars[0].keys)?;

        // The race.
        NO_REPLAY.set(0);
        if !one_car && var(PLAYER_COUNT) != 1 {
            NO_REPLAY.set(1);
        }
        step(5)?; // race::set_up_race_sound_objects
        step(6)?; // sound::set_up_race_sounds
        step(7)?; // race::race_from_menu
        step(2)?; // sound::stop_music
        step(8)?; // gfx::set_menu_screen_mode
        if var(SWITCH) != 1 {
            step(4)?; // menu::show_loading_picture
        }
        step(9)?; // race::clear_race_sound_objects
        step(10)?; // sound::apply_settings

        // The outcome back into the records.
        if one_car {
            let first = player(1)?;
            give_words(first, &FIRST_BACK_A)?;
            copy_bytes(BLOCK_FIRST.addr(), first.wrapping_add(0x24C), 0x20)?;
            set_field(first, 0xD8, total(TIMES_FIRST, EXTRA_FIRST))?;
            give_words(first, &FIRST_BACK_B)?;
            give_outcome(first, &OUTCOME_FIRST)?;
            set_var(CURRENT, 0);
        }
        let rec = player(var(CURRENT))?;
        set_field(rec, 0x284, BACK_284_CURRENT.get())?;
        copy_bytes(BLOCK_CURRENT.addr(), rec.wrapping_add(0x24C), 0x20)?;
        give_words(rec, &SECOND_BACK_A)?;
        set_field(rec, 0xD8, total(TIMES_CURRENT, EXTRA_CURRENT))?;
        give_words(rec, &SECOND_BACK_B)?;
        give_outcome(rec, &OUTCOME_CURRENT)?;

        if var(SWITCH) != 1 {
            step(11)?; // scene::set_up_engine
            step(12)?; // scene::set_up_front_end_scene
            step(11)?;
        }
        step(13)?; // input::take_key
        finish(step)
    }
}

/// The original's call at `place` (its index in `table::CALLS` plus one). The registers the
/// original's calls leave (eax from `show_loading_picture`, eax .. edx from `set_up_engine`,
/// al from `take_key`) are never read.
fn step(place: u32) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::race::rally::copy_rally_times()?,
            2 => crate::sound::stop_music::stop_music(),
            3 => crate::race::race_shell::set_race_screen_mode(),
            4 => {
                crate::menu::show_loading_picture::show_loading_picture()?;
            }
            5 => crate::race::sound_tables::set_up_race_sound_objects(
                &crate::race::sound_tables::ObjectVars::game(),
            )?,
            6 => crate::sound::set_up_race_sounds::set_up_race_sounds()?,
            7 => crate::race::race_end::race_from_menu()?,
            8 => {
                crate::platform::set_menu_mode_now();
            }
            9 => crate::race::sound_tables::clear_race_sound_objects()?,
            10 => crate::sound::apply_settings::apply_settings()?,
            11 => crate::scene::set_up_engine::set_up_engine()?,
            12 => crate::scene::set_up_front_end_scene::set_up_front_end_scene()?,
            13 => {
                crate::input::take_key::take_key();
            }
            _ => crate::sound::play_tune_3::play_tune_3(),
        }
        Ok(())
    }
}

/// The original's instructions as a table and the small interpreter that does them (made from
/// the disassembly by script, used as the game's code until the plain version replaced it):
/// the reference the test compares the plain version with.
#[cfg(test)]
mod table {
    use crate::errors::{FatalError, Refused};
    use crate::framework::{Regs, game_addr, rva};

    /// A register of the table's.
    #[derive(Clone, Copy, Debug)]
    pub enum R {
        Eax,
        Ebx,
        Ecx,
        Edx,
        Esi,
        Edi,
        Ebp,
        /// eax's low byte.
        Al,
    }

    /// One instruction (addresses as IDA shows them; jump targets are op numbers).
    #[derive(Clone, Copy, Debug)]
    pub enum Op {
        Pusha,
        Popa,
        Push(R),
        Pop(R),
        /// `mov [m], r`.
        Store(R, usize),
        /// `mov r, [m]`.
        Load(R, usize),
        /// `mov r, [edi + n]` (al: a byte).
        FromEdi(R, u32),
        /// `mov [edi + n], r`.
        ToEdi(R, u32),
        /// `mov dword [m], value`.
        Put(usize, u32),
        /// `mov dword [edi + n], value`.
        EdiPut(u32, u32),
        /// `mov edi, [table + eax * 4]`.
        EdiFromTable(usize),
        /// `mov r, value`.
        Imm(R, u32),
        /// `lea r, m`.
        Lea(R, usize),
        /// `lea r, [edi + n]`.
        LeaEdi(R, u32),
        /// `mov a, b`.
        Move(R, R),
        Nop,
        /// `cmp dword [m], value`.
        CmpMem(usize, u32),
        /// `cmp r, value`.
        CmpReg(R, u32),
        /// `cmp dword [edi + n], value`.
        CmpEdi(u32, u32),
        /// `cmp [edi + n], eax`.
        CmpEdiEax(u32),
        Jz(usize),
        Jnz(usize),
        Jmp(usize),
        /// `jz` to a fatal stub (0 "Unknown Shaun TrakNum", 1 the division's).
        JzFatal(usize),
        Call(usize),
        /// `rep movsb`.
        RepMovsb,
        /// `add r, [m]`.
        AddMem(R, usize),
        /// `add r, value`.
        AddImm(R, u32),
        /// `add a, b`.
        AddReg(R, R),
        /// `imul r, value`.
        MulImm(R, u32),
        /// `div ebp` (edx:eax).
        DivEbp,
        /// `push eax; fld [esp]; fistp [esp]; pop ebx`: ebx the float eax rounded.
        RoundEaxToEbx,
        Return,
    }

    use Op::*;

    const FATALS: [FatalError; 2] = [
        FatalError {
            code: 0x8073_9281,
            message: "Unknown Shaun TrakNum",
        },
        FatalError {
            code: 0x8070_1010,
            message: "",
        },
    ];

    /// The calls the table makes, by place (its index + 1), as IDA shows them.
    pub const CALLS: [usize; 14] = [
        0x48C997, // race::copy_rally_times
        0x40F826, // sound::stop_music
        0x404830, // race::set_race_screen_mode
        0x47CEA8, // menu::show_loading_picture
        0x415C93, // race::set_up_race_sound_objects
        0x40F02E, // sound::set_up_race_sounds
        0x485570, // race::race_from_menu
        0x404823, // gfx::set_menu_screen_mode (platform)
        0x415C6D, // race::clear_race_sound_objects
        0x40EFF3, // sound::apply_settings
        0x442AF6, // scene::set_up_engine
        0x474E04, // scene::set_up_front_end_scene
        0x407F95, // input::take_key
        0x40F780, // sound::play_tune_3
    ];

    fn mem(a: usize) -> *mut u32 {
        crate::pointers::host(game_addr(rva(a))) as *mut u32
    }

    fn reg(g: &mut Regs, r: R) -> &mut u32 {
        match r {
            R::Eax | R::Al => &mut g.eax,
            R::Ebx => &mut g.ebx,
            R::Ecx => &mut g.ecx,
            R::Edx => &mut g.edx,
            R::Esi => &mut g.esi,
            R::Edi => &mut g.edi,
            R::Ebp => &mut g.ebp,
        }
    }

    /// The table from the first op, with eax and ebx the run's start and kind. The registers
    /// are the table's own scratch (a local `Regs`: the original is `pusha` .. `popa`, it
    /// leaves every register of its caller as it was).
    pub unsafe fn run(
        ops: &[Op],
        start: u32,
        kind: u32,
        step: &mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>,
    ) -> Result<(), Refused> {
        let mut g = Regs {
            eax: start,
            ebx: kind,
            ..Regs::default()
        };
        let mut stack: Vec<u32> = Vec::new();
        let mut saved: Vec<Regs> = Vec::new();
        let mut zero = false;
        let mut pc = 0;
        unsafe {
            loop {
                let op = ops[pc];
                pc += 1;
                match op {
                    Pusha => saved.push(g),
                    Popa => {
                        let s = saved.pop().expect("a popa after its pusha");
                        g = Regs {
                            eflags: g.eflags,
                            ..s
                        };
                    }
                    Push(r) => stack.push(*reg(&mut g, r)),
                    Pop(r) => *reg(&mut g, r) = stack.pop().expect("a pop after its push"),
                    Store(R::Al, a) => *(mem(a) as *mut u8) = g.eax as u8,
                    Store(r, a) => mem(a).write_unaligned(*reg(&mut g, r)),
                    Load(r, a) => *reg(&mut g, r) = mem(a).read_unaligned(),
                    FromEdi(R::Al, n) => {
                        g.eax = g.eax & !0xFF
                            | *(crate::pointers::host(g.edi.wrapping_add(n)) as *const u8) as u32
                    }
                    FromEdi(r, n) => {
                        *reg(&mut g, r) = (crate::pointers::host(g.edi.wrapping_add(n))
                            as *const u32)
                            .read_unaligned()
                    }
                    ToEdi(r, n) => {
                        let v = *reg(&mut g, r);
                        (crate::pointers::host(g.edi.wrapping_add(n)) as *mut u32)
                            .write_unaligned(v)
                    }
                    Put(a, v) => mem(a).write_unaligned(v),
                    EdiPut(n, v) => (crate::pointers::host(g.edi.wrapping_add(n)) as *mut u32)
                        .write_unaligned(v),
                    EdiFromTable(t) => {
                        let at = (game_addr(rva(t))).wrapping_add(g.eax.wrapping_mul(4));
                        g.edi = (crate::pointers::host(at) as *const u32).read_unaligned();
                    }
                    Imm(r, v) => *reg(&mut g, r) = v,
                    Lea(r, a) => *reg(&mut g, r) = game_addr(rva(a)),
                    LeaEdi(r, n) => *reg(&mut g, r) = g.edi.wrapping_add(n),
                    Move(a, b) => *reg(&mut g, a) = *reg(&mut g, b),
                    Nop => {}
                    CmpMem(a, v) => zero = mem(a).read_unaligned() == v,
                    CmpReg(r, v) => zero = *reg(&mut g, r) == v,
                    CmpEdi(n, v) => {
                        zero = (crate::pointers::host(g.edi.wrapping_add(n)) as *const u32)
                            .read_unaligned()
                            == v
                    }
                    CmpEdiEax(n) => {
                        zero = (crate::pointers::host(g.edi.wrapping_add(n)) as *const u32)
                            .read_unaligned()
                            == g.eax
                    }
                    Jz(t) => {
                        if zero {
                            pc = t
                        }
                    }
                    Jnz(t) => {
                        if !zero {
                            pc = t
                        }
                    }
                    Jmp(t) => pc = t,
                    JzFatal(n) => {
                        if zero {
                            return Err(FATALS[n].into());
                        }
                    }
                    Call(a) => {
                        let n = CALLS
                            .iter()
                            .position(|&c| c == a)
                            .expect("a call the table has");
                        step(n as u32 + 1, &mut g)?;
                    }
                    RepMovsb => {
                        std::ptr::copy(
                            crate::pointers::host(g.esi) as *const u8,
                            crate::pointers::host(g.edi),
                            g.ecx as usize,
                        );
                        g.esi = g.esi.wrapping_add(g.ecx);
                        g.edi = g.edi.wrapping_add(g.ecx);
                        g.ecx = 0;
                    }
                    AddMem(r, a) => {
                        let v = mem(a).read_unaligned();
                        let x = reg(&mut g, r);
                        *x = x.wrapping_add(v);
                    }
                    AddImm(r, v) => {
                        let x = reg(&mut g, r);
                        *x = x.wrapping_add(v);
                    }
                    AddReg(a, b) => {
                        let v = *reg(&mut g, b);
                        let x = reg(&mut g, a);
                        *x = x.wrapping_add(v);
                    }
                    MulImm(r, v) => {
                        let x = reg(&mut g, r);
                        *x = x.wrapping_mul(v);
                    }
                    DivEbp => {
                        let n = ((g.edx as u64) << 32) | g.eax as u64;
                        let q = n / g.ebp as u64;
                        g.edx = (n % g.ebp as u64) as u32;
                        g.eax = q as u32;
                    }
                    RoundEaxToEbx => g.ebx = crate::math::x87::to_int(g.eax) as u32,
                    Return => return Ok(()),
                }
            }
        }
    }

    // The table (made from the disassembly by script).
    pub const SINGLE_RALLY: [Op; 902] = [
        Pusha,
        CmpReg(R::Ebx, 0x3),
        Jnz(4),
        Call(0x48C997),
        Load(R::Esi, 0x7556F4),
        Store(R::Esi, 0x75ACD0),
        Store(R::Eax, 0x7587E8),
        Store(R::Ebx, 0x75ACD8),
        Put(0x75B028, 0x0),
        Put(0x75B098, 0x0),
        Put(0x75ACE4, 0x0),
        Put(0x75ACE0, 0x1),
        Call(0x40F826),
        Call(0x404830),
        Call(0x47CEA8),
        Load(R::Eax, 0x7558DC),
        Store(R::Eax, 0x5C2FD0),
        Lea(R::Esi, 0x75ACD0),
        Load(R::Eax, 0x75ACD4),
        Lea(R::Edi, 0x734BC6),
        AddImm(R::Edi, 0x34),
        CmpEdiEax(0x34),
        Jz(27),
        AddImm(R::Edi, 0x40),
        CmpEdi(0x0, 0xFFFFFFFF),
        JzFatal(0),
        Jmp(21),
        FromEdi(R::Eax, 0x1C),
        Store(R::Eax, 0x75ACF8),
        FromEdi(R::Eax, 0x20),
        Store(R::Eax, 0x75ACFC),
        Load(R::Eax, 0x755650),
        Store(R::Eax, 0x75AD00),
        Load(R::Eax, 0x75565C),
        Store(R::Eax, 0x75AD0C),
        Load(R::Eax, 0x755660),
        Store(R::Eax, 0x75AD10),
        Load(R::Eax, 0x755664),
        Store(R::Eax, 0x75AD14),
        Load(R::Eax, 0x755668),
        Store(R::Eax, 0x75AD18),
        Push(R::Edi),
        Push(R::Eax),
        Imm(R::Eax, 0x1),
        EdiFromTable(0x75647C),
        Pop(R::Eax),
        FromEdi(R::Eax, 0x2C),
        Store(R::Eax, 0x75AF94),
        FromEdi(R::Eax, 0x30),
        Store(R::Eax, 0x75AF98),
        FromEdi(R::Eax, 0x34),
        Store(R::Eax, 0x75AF9C),
        FromEdi(R::Eax, 0x38),
        Store(R::Eax, 0x75AFA0),
        FromEdi(R::Eax, 0x3C),
        Store(R::Eax, 0x75AFA4),
        FromEdi(R::Eax, 0x40),
        Store(R::Eax, 0x75AFA8),
        FromEdi(R::Eax, 0x44),
        Store(R::Eax, 0x75AFAC),
        FromEdi(R::Eax, 0x48),
        Store(R::Eax, 0x75AFB0),
        FromEdi(R::Eax, 0x50),
        Store(R::Eax, 0x75AFB8),
        FromEdi(R::Eax, 0x54),
        Store(R::Eax, 0x75AFBC),
        FromEdi(R::Eax, 0x58),
        Store(R::Eax, 0x75AFC0),
        FromEdi(R::Eax, 0x5C),
        Store(R::Eax, 0x75AFC4),
        FromEdi(R::Eax, 0x60),
        Store(R::Eax, 0x75AFC8),
        FromEdi(R::Eax, 0x64),
        Store(R::Eax, 0x75AFCC),
        FromEdi(R::Eax, 0x68),
        Store(R::Eax, 0x75AFD0),
        FromEdi(R::Eax, 0x6C),
        Store(R::Eax, 0x75AFD4),
        FromEdi(R::Eax, 0x70),
        Store(R::Eax, 0x75AFD8),
        FromEdi(R::Eax, 0x74),
        Store(R::Eax, 0x75AFE0),
        FromEdi(R::Eax, 0x78),
        Store(R::Eax, 0x75AFE4),
        FromEdi(R::Al, 0x1C),
        Store(R::Al, 0x75AFE8),
        FromEdi(R::Al, 0x1D),
        Store(R::Al, 0x75AFE9),
        FromEdi(R::Al, 0x1E),
        Store(R::Al, 0x75AFEA),
        FromEdi(R::Al, 0x1F),
        Store(R::Al, 0x75AFEB),
        FromEdi(R::Al, 0x20),
        Store(R::Al, 0x75AFEC),
        FromEdi(R::Al, 0x21),
        Store(R::Al, 0x75AFED),
        FromEdi(R::Al, 0x22),
        Store(R::Al, 0x75AFEE),
        FromEdi(R::Al, 0x23),
        Store(R::Al, 0x75AFEF),
        FromEdi(R::Al, 0x24),
        Store(R::Al, 0x75AFF0),
        FromEdi(R::Al, 0x25),
        Store(R::Al, 0x75AFF1),
        FromEdi(R::Al, 0x26),
        Store(R::Al, 0x75AFF2),
        FromEdi(R::Al, 0x27),
        Store(R::Al, 0x75AFF3),
        Pop(R::Edi),
        CmpMem(0x755988, 0x5),
        Jz(126),
        CmpMem(0x755988, 0x4),
        Jz(126),
        CmpMem(0x755988, 0xF),
        Jz(126),
        CmpMem(0x755988, 0xA),
        Jz(126),
        Push(R::Eax),
        Load(R::Eax, 0x75598C),
        EdiFromTable(0x75647C),
        Pop(R::Eax),
        CmpEdi(0xD0, 0x1),
        Jz(290),
        Put(0x75B028, 0xFFFFFFFF),
        Jmp(897),
        Jmp(290),
        Load(R::Edx, 0x7554C8),
        Store(R::Edx, 0x75ACE4),
        Put(0x75ACE0, 0x2),
        Push(R::Eax),
        Imm(R::Eax, 0x1),
        EdiFromTable(0x75647C),
        Pop(R::Eax),
        CmpEdi(0xD0, 0x1),
        Jz(136),
        Put(0x75B098, 0xFFFFFFFF),
        FromEdi(R::Eax, 0x8),
        Store(R::Eax, 0x75ACEC),
        Load(R::Eax, 0x75ACEC),
        FromEdi(R::Eax, 0x10C),
        Store(R::Eax, 0x75AE88),
        FromEdi(R::Eax, 0x110),
        Store(R::Eax, 0x75AE8C),
        FromEdi(R::Eax, 0x114),
        Store(R::Eax, 0x75AE90),
        FromEdi(R::Eax, 0x118),
        Store(R::Eax, 0x75AE94),
        FromEdi(R::Eax, 0x11C),
        Store(R::Eax, 0x75AE98),
        FromEdi(R::Eax, 0x120),
        Store(R::Eax, 0x75AE9C),
        FromEdi(R::Eax, 0x288),
        Store(R::Eax, 0x75B008),
        Pusha,
        Imm(R::Ecx, 0x20),
        LeaEdi(R::Esi, 0x24C),
        Lea(R::Edi, 0x75B0DC),
        RepMovsb,
        Popa,
        FromEdi(R::Eax, 0x278),
        Store(R::Eax, 0x75B018),
        FromEdi(R::Eax, 0xD4),
        Store(R::Eax, 0x75AEA0),
        FromEdi(R::Eax, 0xD8),
        Store(R::Eax, 0x75AEA4),
        FromEdi(R::Eax, 0xDC),
        Store(R::Eax, 0x75AEA8),
        FromEdi(R::Eax, 0xE0),
        Store(R::Eax, 0x75AEAC),
        FromEdi(R::Eax, 0xE4),
        Store(R::Eax, 0x75AEB0),
        FromEdi(R::Eax, 0xE8),
        Store(R::Eax, 0x75AEB4),
        FromEdi(R::Eax, 0xEC),
        Store(R::Eax, 0x75AEB8),
        FromEdi(R::Eax, 0xF0),
        Store(R::Eax, 0x75AEBC),
        FromEdi(R::Eax, 0xF4),
        Store(R::Eax, 0x75AEC0),
        FromEdi(R::Eax, 0xF8),
        Store(R::Eax, 0x75AEC4),
        FromEdi(R::Eax, 0xFC),
        Store(R::Eax, 0x75AEC8),
        FromEdi(R::Eax, 0x100),
        Store(R::Eax, 0x75AECC),
        FromEdi(R::Eax, 0x104),
        Store(R::Eax, 0x75AEF8),
        FromEdi(R::Eax, 0x108),
        Store(R::Eax, 0x75AED0),
        FromEdi(R::Eax, 0x158),
        Store(R::Eax, 0x75AF64),
        FromEdi(R::Eax, 0x15C),
        Store(R::Eax, 0x75AF68),
        FromEdi(R::Eax, 0x160),
        Store(R::Eax, 0x75AF6C),
        FromEdi(R::Eax, 0x168),
        Store(R::Eax, 0x75AF74),
        FromEdi(R::Eax, 0x164),
        Store(R::Eax, 0x75AF78),
        FromEdi(R::Eax, 0x170),
        Store(R::Eax, 0x75AF80),
        FromEdi(R::Eax, 0x16C),
        Store(R::Eax, 0x75AF7C),
        FromEdi(R::Eax, 0x174),
        Store(R::Eax, 0x75AF84),
        FromEdi(R::Eax, 0x178),
        Store(R::Eax, 0x75AF70),
        FromEdi(R::Eax, 0x284),
        Store(R::Eax, 0x75B010),
        FromEdi(R::Eax, 0x17C),
        Store(R::Eax, 0x75AED4),
        FromEdi(R::Eax, 0x180),
        Store(R::Eax, 0x75AED8),
        FromEdi(R::Eax, 0x184),
        Store(R::Eax, 0x75AEDC),
        FromEdi(R::Eax, 0x188),
        Store(R::Eax, 0x75AEE0),
        FromEdi(R::Eax, 0x18C),
        Store(R::Eax, 0x75AEE4),
        FromEdi(R::Eax, 0x190),
        Store(R::Eax, 0x75AEE8),
        FromEdi(R::Eax, 0x194),
        Store(R::Eax, 0x75AEEC),
        FromEdi(R::Eax, 0x198),
        Store(R::Eax, 0x75AEF0),
        FromEdi(R::Eax, 0x19C),
        Store(R::Eax, 0x75AEF4),
        FromEdi(R::Eax, 0x1A0),
        Store(R::Eax, 0x75AEF8),
        FromEdi(R::Eax, 0x1A4),
        Store(R::Eax, 0x75AEFC),
        FromEdi(R::Eax, 0x1A8),
        Store(R::Eax, 0x75AF00),
        FromEdi(R::Eax, 0x1AC),
        Store(R::Eax, 0x75AF04),
        FromEdi(R::Eax, 0x1B0),
        Store(R::Eax, 0x75AF08),
        FromEdi(R::Eax, 0x1B4),
        Store(R::Eax, 0x75AF0C),
        FromEdi(R::Eax, 0x1B8),
        Store(R::Eax, 0x75AF10),
        FromEdi(R::Eax, 0x1BC),
        Store(R::Eax, 0x75AF14),
        FromEdi(R::Eax, 0x1C0),
        Store(R::Eax, 0x75AF18),
        FromEdi(R::Eax, 0x1C4),
        Store(R::Eax, 0x75AF1C),
        FromEdi(R::Eax, 0x1C8),
        Store(R::Eax, 0x75AF20),
        FromEdi(R::Eax, 0x1CC),
        Store(R::Eax, 0x75AF24),
        FromEdi(R::Eax, 0x1D0),
        Store(R::Eax, 0x75AF28),
        FromEdi(R::Eax, 0x1D4),
        Store(R::Eax, 0x75AF2C),
        FromEdi(R::Eax, 0x1D8),
        Store(R::Eax, 0x75AF30),
        FromEdi(R::Eax, 0x1DC),
        Store(R::Eax, 0x75AF34),
        FromEdi(R::Eax, 0x1E0),
        Store(R::Eax, 0x75AF38),
        FromEdi(R::Eax, 0x1E4),
        Store(R::Eax, 0x75AF3C),
        FromEdi(R::Eax, 0x1E8),
        Store(R::Eax, 0x75AF40),
        FromEdi(R::Eax, 0x1EC),
        Store(R::Eax, 0x75AF44),
        FromEdi(R::Eax, 0x1F0),
        Store(R::Eax, 0x75AF48),
        FromEdi(R::Eax, 0x1F4),
        Store(R::Eax, 0x75AF4C),
        FromEdi(R::Eax, 0x1F8),
        Store(R::Eax, 0x75AF50),
        FromEdi(R::Eax, 0x1FC),
        Store(R::Eax, 0x75AF54),
        FromEdi(R::Eax, 0x200),
        Store(R::Eax, 0x75AF58),
        FromEdi(R::Eax, 0x204),
        Store(R::Eax, 0x75AF5C),
        FromEdi(R::Eax, 0x208),
        Store(R::Eax, 0x75AF60),
        FromEdi(R::Eax, 0x28),
        Store(R::Eax, 0x75AF88),
        FromEdi(R::Eax, 0x154),
        Store(R::Eax, 0x75AF8C),
        FromEdi(R::Eax, 0xC),
        Store(R::Eax, 0x75AF90),
        FromEdi(R::Eax, 0x10),
        Store(R::Eax, 0x75AF90),
        Put(0x75598C, 0x0),
        Push(R::Eax),
        Load(R::Eax, 0x75598C),
        EdiFromTable(0x75647C),
        Pop(R::Eax),
        FromEdi(R::Eax, 0x8),
        Store(R::Eax, 0x75ACE8),
        Load(R::Eax, 0x75ACE8),
        CmpEdi(0xD0, 0x1),
        Jz(300),
        Put(0x75B028, 0xFFFFFFFF),
        FromEdi(R::Eax, 0x10C),
        Store(R::Eax, 0x75AD1C),
        FromEdi(R::Eax, 0x110),
        Store(R::Eax, 0x75AD20),
        FromEdi(R::Eax, 0x114),
        Store(R::Eax, 0x75AD24),
        FromEdi(R::Eax, 0x118),
        Store(R::Eax, 0x75AD28),
        FromEdi(R::Eax, 0x11C),
        Store(R::Eax, 0x75AD2C),
        FromEdi(R::Eax, 0x120),
        Store(R::Eax, 0x75AD30),
        FromEdi(R::Eax, 0x288),
        Store(R::Eax, 0x75B004),
        Pusha,
        Imm(R::Ecx, 0x20),
        LeaEdi(R::Esi, 0x24C),
        Lea(R::Edi, 0x75B06C),
        RepMovsb,
        Popa,
        FromEdi(R::Eax, 0x278),
        Store(R::Eax, 0x75B014),
        FromEdi(R::Eax, 0xD4),
        Store(R::Eax, 0x75AD34),
        FromEdi(R::Eax, 0xD8),
        Store(R::Eax, 0x75AD38),
        FromEdi(R::Eax, 0xDC),
        Store(R::Eax, 0x75AD3C),
        FromEdi(R::Eax, 0xE0),
        Store(R::Eax, 0x75AD40),
        FromEdi(R::Eax, 0xE4),
        Store(R::Eax, 0x75AD44),
        FromEdi(R::Eax, 0xE8),
        Store(R::Eax, 0x75AD48),
        FromEdi(R::Eax, 0xEC),
        Store(R::Eax, 0x75AD4C),
        FromEdi(R::Eax, 0xF0),
        Store(R::Eax, 0x75AD50),
        FromEdi(R::Eax, 0xF4),
        Store(R::Eax, 0x75AD54),
        FromEdi(R::Eax, 0xF8),
        Store(R::Eax, 0x75AD58),
        FromEdi(R::Eax, 0xFC),
        Store(R::Eax, 0x75AD5C),
        FromEdi(R::Eax, 0x100),
        Store(R::Eax, 0x75AD60),
        FromEdi(R::Eax, 0x104),
        Store(R::Eax, 0x75AD8C),
        FromEdi(R::Eax, 0x108),
        Store(R::Eax, 0x75AD64),
        FromEdi(R::Eax, 0x158),
        Store(R::Eax, 0x75ADF8),
        FromEdi(R::Eax, 0x15C),
        Store(R::Eax, 0x75ADFC),
        FromEdi(R::Eax, 0x160),
        Store(R::Eax, 0x75AE00),
        FromEdi(R::Eax, 0x168),
        Store(R::Eax, 0x75AE08),
        FromEdi(R::Eax, 0x164),
        Store(R::Eax, 0x75AE0C),
        FromEdi(R::Eax, 0x170),
        Store(R::Eax, 0x75AE14),
        FromEdi(R::Eax, 0x16C),
        Store(R::Eax, 0x75AE10),
        FromEdi(R::Eax, 0x174),
        Store(R::Eax, 0x75AE18),
        FromEdi(R::Eax, 0x178),
        Store(R::Eax, 0x75AE04),
        FromEdi(R::Eax, 0x284),
        Store(R::Eax, 0x75B00C),
        FromEdi(R::Eax, 0x17C),
        Store(R::Eax, 0x75AD68),
        FromEdi(R::Eax, 0x180),
        Store(R::Eax, 0x75AD6C),
        FromEdi(R::Eax, 0x184),
        Store(R::Eax, 0x75AD70),
        FromEdi(R::Eax, 0x188),
        Store(R::Eax, 0x75AD74),
        FromEdi(R::Eax, 0x18C),
        Store(R::Eax, 0x75AD78),
        FromEdi(R::Eax, 0x190),
        Store(R::Eax, 0x75AD7C),
        FromEdi(R::Eax, 0x194),
        Store(R::Eax, 0x75AD80),
        FromEdi(R::Eax, 0x198),
        Store(R::Eax, 0x75AD84),
        FromEdi(R::Eax, 0x19C),
        Store(R::Eax, 0x75AD88),
        FromEdi(R::Eax, 0x1A0),
        Store(R::Eax, 0x75AD8C),
        FromEdi(R::Eax, 0x1A4),
        Store(R::Eax, 0x75AD90),
        FromEdi(R::Eax, 0x1A8),
        Store(R::Eax, 0x75AD94),
        FromEdi(R::Eax, 0x1AC),
        Store(R::Eax, 0x75AD98),
        FromEdi(R::Eax, 0x1B0),
        Store(R::Eax, 0x75AD9C),
        FromEdi(R::Eax, 0x1B4),
        Store(R::Eax, 0x75ADA0),
        FromEdi(R::Eax, 0x1B8),
        Store(R::Eax, 0x75ADA4),
        FromEdi(R::Eax, 0x1BC),
        Store(R::Eax, 0x75ADA8),
        FromEdi(R::Eax, 0x1C0),
        Store(R::Eax, 0x75ADAC),
        FromEdi(R::Eax, 0x1C4),
        Store(R::Eax, 0x75ADB0),
        FromEdi(R::Eax, 0x1C8),
        Store(R::Eax, 0x75ADB4),
        FromEdi(R::Eax, 0x1CC),
        Store(R::Eax, 0x75ADB8),
        FromEdi(R::Eax, 0x1D0),
        Store(R::Eax, 0x75ADBC),
        FromEdi(R::Eax, 0x1D4),
        Store(R::Eax, 0x75ADC0),
        FromEdi(R::Eax, 0x1D8),
        Store(R::Eax, 0x75ADC4),
        FromEdi(R::Eax, 0x1DC),
        Store(R::Eax, 0x75ADC8),
        FromEdi(R::Eax, 0x1E0),
        Store(R::Eax, 0x75ADCC),
        FromEdi(R::Eax, 0x1E4),
        Store(R::Eax, 0x75ADD0),
        FromEdi(R::Eax, 0x1E8),
        Store(R::Eax, 0x75ADD4),
        FromEdi(R::Eax, 0x1EC),
        Store(R::Eax, 0x75ADD8),
        FromEdi(R::Eax, 0x1F0),
        Store(R::Eax, 0x75ADDC),
        FromEdi(R::Eax, 0x1F4),
        Store(R::Eax, 0x75ADE0),
        FromEdi(R::Eax, 0x1F8),
        Store(R::Eax, 0x75ADE4),
        FromEdi(R::Eax, 0x1FC),
        Store(R::Eax, 0x75ADE8),
        FromEdi(R::Eax, 0x200),
        Store(R::Eax, 0x75ADEC),
        FromEdi(R::Eax, 0x204),
        Store(R::Eax, 0x75ADF0),
        FromEdi(R::Eax, 0x208),
        Store(R::Eax, 0x75ADF4),
        FromEdi(R::Eax, 0x28),
        Store(R::Eax, 0x75AE1C),
        FromEdi(R::Eax, 0x154),
        Store(R::Eax, 0x75AE20),
        FromEdi(R::Eax, 0xC),
        Store(R::Eax, 0x75AE24),
        CmpMem(0x75ACE0, 0x2),
        Jnz(452),
        FromEdi(R::Eax, 0x10),
        Store(R::Eax, 0x75AE24),
        FromEdi(R::Eax, 0x2C),
        Store(R::Eax, 0x75AE28),
        FromEdi(R::Eax, 0x30),
        Store(R::Eax, 0x75AE2C),
        FromEdi(R::Eax, 0x34),
        Store(R::Eax, 0x75AE30),
        FromEdi(R::Eax, 0x38),
        Store(R::Eax, 0x75AE34),
        FromEdi(R::Eax, 0x3C),
        Store(R::Eax, 0x75AE38),
        FromEdi(R::Eax, 0x40),
        Store(R::Eax, 0x75AE3C),
        FromEdi(R::Eax, 0x44),
        Store(R::Eax, 0x75AE40),
        FromEdi(R::Eax, 0x48),
        Store(R::Eax, 0x75AE44),
        FromEdi(R::Eax, 0x50),
        Store(R::Eax, 0x75AE4C),
        FromEdi(R::Eax, 0x54),
        Store(R::Eax, 0x75AE50),
        FromEdi(R::Eax, 0x58),
        Store(R::Eax, 0x75AE54),
        FromEdi(R::Eax, 0x5C),
        Store(R::Eax, 0x75AE58),
        FromEdi(R::Eax, 0x60),
        Store(R::Eax, 0x75AE5C),
        FromEdi(R::Eax, 0x64),
        Store(R::Eax, 0x75AE60),
        FromEdi(R::Eax, 0x68),
        Store(R::Eax, 0x75AE64),
        FromEdi(R::Eax, 0x6C),
        Store(R::Eax, 0x75AE68),
        FromEdi(R::Eax, 0x70),
        Store(R::Eax, 0x75AE6C),
        FromEdi(R::Eax, 0x74),
        Store(R::Eax, 0x75AE74),
        FromEdi(R::Eax, 0x78),
        Store(R::Eax, 0x75AE78),
        FromEdi(R::Al, 0x1C),
        Store(R::Al, 0x75AE7C),
        FromEdi(R::Al, 0x1D),
        Store(R::Al, 0x75AE7D),
        FromEdi(R::Al, 0x1E),
        Store(R::Al, 0x75AE7E),
        FromEdi(R::Al, 0x1F),
        Store(R::Al, 0x75AE7F),
        FromEdi(R::Al, 0x20),
        Store(R::Al, 0x75AE80),
        FromEdi(R::Al, 0x21),
        Store(R::Al, 0x75AE81),
        FromEdi(R::Al, 0x22),
        Store(R::Al, 0x75AE82),
        FromEdi(R::Al, 0x23),
        Store(R::Al, 0x75AE83),
        FromEdi(R::Al, 0x24),
        Store(R::Al, 0x75AE84),
        FromEdi(R::Al, 0x25),
        Store(R::Al, 0x75AE85),
        FromEdi(R::Al, 0x26),
        Store(R::Al, 0x75AE86),
        FromEdi(R::Al, 0x27),
        Store(R::Al, 0x75AE87),
        Put(0x75ACDC, 0x0),
        CmpMem(0x755988, 0x5),
        Jz(526),
        CmpMem(0x755988, 0x4),
        Jz(526),
        CmpMem(0x755988, 0xF),
        Jz(526),
        CmpMem(0x755988, 0xA),
        Jz(526),
        CmpMem(0x755984, 0x1),
        Jz(526),
        Put(0x75ACDC, 0x1),
        Call(0x415C93),
        Call(0x40F02E),
        Call(0x485570),
        Call(0x40F826),
        Call(0x404823),
        Load(R::Eax, 0x7587E8),
        CmpReg(R::Eax, 0x1),
        Jz(535),
        Call(0x47CEA8),
        Call(0x415C6D),
        Call(0x40EFF3),
        Lea(R::Esi, 0x75ACD0),
        CmpMem(0x755988, 0x5),
        Jz(547),
        CmpMem(0x755988, 0x4),
        Jz(547),
        CmpMem(0x755988, 0xF),
        Jz(547),
        CmpMem(0x755988, 0xA),
        Jz(547),
        Jmp(720),
        Push(R::Eax),
        Imm(R::Eax, 0x1),
        EdiFromTable(0x75647C),
        Pop(R::Eax),
        Load(R::Eax, 0x75B010),
        ToEdi(R::Eax, 0x284),
        Load(R::Eax, 0x75B018),
        ToEdi(R::Eax, 0x278),
        Load(R::Eax, 0x75AEA0),
        ToEdi(R::Eax, 0xD4),
        Load(R::Eax, 0x75AEA4),
        ToEdi(R::Eax, 0xD8),
        Load(R::Eax, 0x75AEA8),
        ToEdi(R::Eax, 0xDC),
        Load(R::Eax, 0x75AEAC),
        ToEdi(R::Eax, 0xE0),
        Load(R::Eax, 0x75AEB0),
        ToEdi(R::Eax, 0xE4),
        Load(R::Eax, 0x75AEB4),
        ToEdi(R::Eax, 0xE8),
        Load(R::Eax, 0x75AEB8),
        ToEdi(R::Eax, 0xEC),
        Load(R::Eax, 0x75AEBC),
        ToEdi(R::Eax, 0xF0),
        Load(R::Eax, 0x75AEC0),
        ToEdi(R::Eax, 0xF4),
        Load(R::Eax, 0x75AEC4),
        ToEdi(R::Eax, 0xF8),
        Load(R::Eax, 0x75AEC8),
        ToEdi(R::Eax, 0xFC),
        Load(R::Eax, 0x75AECC),
        ToEdi(R::Eax, 0x100),
        Load(R::Eax, 0x75AEF8),
        ToEdi(R::Eax, 0x104),
        Load(R::Eax, 0x75AED0),
        ToEdi(R::Eax, 0x108),
        Load(R::Eax, 0x75AED4),
        ToEdi(R::Eax, 0x17C),
        Load(R::Eax, 0x75AED8),
        ToEdi(R::Eax, 0x180),
        Load(R::Eax, 0x75AEDC),
        ToEdi(R::Eax, 0x184),
        Load(R::Eax, 0x75AEE0),
        ToEdi(R::Eax, 0x188),
        Load(R::Eax, 0x75AEE4),
        ToEdi(R::Eax, 0x18C),
        Load(R::Eax, 0x75AEE8),
        ToEdi(R::Eax, 0x190),
        Load(R::Eax, 0x75AEEC),
        ToEdi(R::Eax, 0x194),
        Load(R::Eax, 0x75AEF0),
        ToEdi(R::Eax, 0x198),
        Load(R::Eax, 0x75AEF4),
        ToEdi(R::Eax, 0x19C),
        Load(R::Eax, 0x75AEF8),
        ToEdi(R::Eax, 0x1A0),
        Load(R::Eax, 0x75AEFC),
        ToEdi(R::Eax, 0x1A4),
        Load(R::Eax, 0x75AF00),
        ToEdi(R::Eax, 0x1A8),
        Load(R::Eax, 0x75AF04),
        ToEdi(R::Eax, 0x1AC),
        Load(R::Eax, 0x75AF08),
        ToEdi(R::Eax, 0x1B0),
        Load(R::Eax, 0x75AF0C),
        ToEdi(R::Eax, 0x1B4),
        Load(R::Eax, 0x75AF10),
        ToEdi(R::Eax, 0x1B8),
        Load(R::Eax, 0x75AF14),
        ToEdi(R::Eax, 0x1BC),
        Load(R::Eax, 0x75AF18),
        ToEdi(R::Eax, 0x1C0),
        Load(R::Eax, 0x75AF1C),
        ToEdi(R::Eax, 0x1C4),
        Load(R::Eax, 0x75AF20),
        ToEdi(R::Eax, 0x1C8),
        Load(R::Eax, 0x75AF24),
        ToEdi(R::Eax, 0x1CC),
        Load(R::Eax, 0x75AF28),
        ToEdi(R::Eax, 0x1D0),
        Load(R::Eax, 0x75AF2C),
        ToEdi(R::Eax, 0x1D4),
        Load(R::Eax, 0x75AF30),
        ToEdi(R::Eax, 0x1D8),
        Load(R::Eax, 0x75AF34),
        ToEdi(R::Eax, 0x1DC),
        Load(R::Eax, 0x75AF38),
        ToEdi(R::Eax, 0x1E0),
        Load(R::Eax, 0x75AF3C),
        ToEdi(R::Eax, 0x1E4),
        Load(R::Eax, 0x75AF40),
        ToEdi(R::Eax, 0x1E8),
        Load(R::Eax, 0x75AF44),
        ToEdi(R::Eax, 0x1EC),
        Load(R::Eax, 0x75AF48),
        ToEdi(R::Eax, 0x1F0),
        Load(R::Eax, 0x75AF4C),
        ToEdi(R::Eax, 0x1F4),
        Load(R::Eax, 0x75AF50),
        ToEdi(R::Eax, 0x1F8),
        Load(R::Eax, 0x75AF54),
        ToEdi(R::Eax, 0x1FC),
        Load(R::Eax, 0x75AF58),
        ToEdi(R::Eax, 0x200),
        Load(R::Eax, 0x75AF5C),
        ToEdi(R::Eax, 0x204),
        Load(R::Eax, 0x75AF60),
        ToEdi(R::Eax, 0x208),
        Pusha,
        Imm(R::Ecx, 0x20),
        Lea(R::Esi, 0x75B0DC),
        LeaEdi(R::Edi, 0x24C),
        RepMovsb,
        Popa,
        Load(R::Ebx, 0x75AEFC),
        AddMem(R::Ebx, 0x75AF00),
        AddMem(R::Ebx, 0x75AF04),
        AddMem(R::Ebx, 0x75AF08),
        CmpReg(R::Ebx, 0x0),
        Jz(674),
        Move(R::Eax, R::Ebx),
        CmpReg(R::Eax, 0x0),
        JzFatal(1),
        Imm(R::Edx, 0),
        Imm(R::Ebp, 0xA),
        DivEbp,
        Move(R::Ebx, R::Eax),
        Load(R::Eax, 0x75AF0C),
        MulImm(R::Eax, 0x6),
        CmpReg(R::Eax, 0x0),
        Jz(684),
        Nop,
        CmpReg(R::Eax, 0x0),
        JzFatal(1),
        Imm(R::Edx, 0),
        Imm(R::Ebp, 0xA),
        DivEbp,
        AddReg(R::Ebx, R::Eax),
        ToEdi(R::Ebx, 0xD8),
        Load(R::Eax, 0x75AF40),
        ToEdi(R::Eax, 0xF8),
        Load(R::Eax, 0x75AF60),
        ToEdi(R::Eax, 0x108),
        Load(R::Eax, 0x75AF4C),
        ToEdi(R::Eax, 0x100),
        Load(R::Eax, 0x75AF2C),
        ToEdi(R::Eax, 0xE8),
        Load(R::Eax, 0x75AF8C),
        ToEdi(R::Eax, 0x154),
        Load(R::Eax, 0x75AF90),
        ToEdi(R::Eax, 0xC),
        CmpMem(0x75B098, 0xFFFFFFFE),
        Jz(702),
        CmpMem(0x75B098, 0xFFFFFFFF),
        Jnz(705),
        EdiPut(0x88, 0x36EE80),
        EdiPut(0xD0, 0x6),
        Jmp(719),
        Load(R::Eax, 0x75B08C),
        ToEdi(R::Eax, 0x88),
        Load(R::Eax, 0x75B090),
        ToEdi(R::Eax, 0x27C),
        Load(R::Eax, 0x75B094),
        RoundEaxToEbx,
        MulImm(R::Ebx, 0xA),
        ToEdi(R::Ebx, 0x94),
        Pusha,
        Imm(R::Ecx, 0x40),
        Lea(R::Esi, 0x75B09C),
        LeaEdi(R::Edi, 0x20C),
        RepMovsb,
        Popa,
        Put(0x75598C, 0x0),
        Push(R::Eax),
        Load(R::Eax, 0x75598C),
        EdiFromTable(0x75647C),
        Pop(R::Eax),
        Load(R::Eax, 0x75B00C),
        ToEdi(R::Eax, 0x284),
        Pusha,
        Imm(R::Ecx, 0x20),
        Lea(R::Esi, 0x75B06C),
        LeaEdi(R::Edi, 0x24C),
        RepMovsb,
        Popa,
        Load(R::Eax, 0x75B014),
        ToEdi(R::Eax, 0x278),
        Load(R::Eax, 0x75AD34),
        ToEdi(R::Eax, 0xD4),
        Load(R::Eax, 0x75AD38),
        ToEdi(R::Eax, 0xD8),
        Load(R::Eax, 0x75AD3C),
        ToEdi(R::Eax, 0xDC),
        Load(R::Eax, 0x75AD40),
        ToEdi(R::Eax, 0xE0),
        Load(R::Eax, 0x75AD44),
        ToEdi(R::Eax, 0xE4),
        Load(R::Eax, 0x75AD48),
        ToEdi(R::Eax, 0xE8),
        Load(R::Eax, 0x75AD4C),
        ToEdi(R::Eax, 0xEC),
        Load(R::Eax, 0x75AD50),
        ToEdi(R::Eax, 0xF0),
        Load(R::Eax, 0x75AD54),
        ToEdi(R::Eax, 0xF4),
        Load(R::Eax, 0x75AD5C),
        ToEdi(R::Eax, 0xFC),
        Load(R::Eax, 0x75AD60),
        ToEdi(R::Eax, 0x100),
        Load(R::Eax, 0x75AD8C),
        ToEdi(R::Eax, 0x104),
        Load(R::Eax, 0x75AD64),
        ToEdi(R::Eax, 0x108),
        Load(R::Eax, 0x75AD68),
        ToEdi(R::Eax, 0x17C),
        Load(R::Eax, 0x75AD6C),
        ToEdi(R::Eax, 0x180),
        Load(R::Eax, 0x75AD70),
        ToEdi(R::Eax, 0x184),
        Load(R::Eax, 0x75AD74),
        ToEdi(R::Eax, 0x188),
        Load(R::Eax, 0x75AD78),
        ToEdi(R::Eax, 0x18C),
        Load(R::Eax, 0x75AD7C),
        ToEdi(R::Eax, 0x190),
        Load(R::Eax, 0x75AD80),
        ToEdi(R::Eax, 0x194),
        Load(R::Eax, 0x75AD84),
        ToEdi(R::Eax, 0x198),
        Load(R::Eax, 0x75AD88),
        ToEdi(R::Eax, 0x19C),
        Load(R::Eax, 0x75AD8C),
        ToEdi(R::Eax, 0x1A0),
        Load(R::Eax, 0x75AD90),
        ToEdi(R::Eax, 0x1A4),
        Load(R::Eax, 0x75AD94),
        ToEdi(R::Eax, 0x1A8),
        Load(R::Eax, 0x75AD98),
        ToEdi(R::Eax, 0x1AC),
        Load(R::Eax, 0x75AD9C),
        ToEdi(R::Eax, 0x1B0),
        Load(R::Eax, 0x75ADA0),
        ToEdi(R::Eax, 0x1B4),
        Load(R::Eax, 0x75ADA4),
        ToEdi(R::Eax, 0x1B8),
        Load(R::Eax, 0x75ADA8),
        ToEdi(R::Eax, 0x1BC),
        Load(R::Eax, 0x75ADAC),
        ToEdi(R::Eax, 0x1C0),
        Load(R::Eax, 0x75ADB0),
        ToEdi(R::Eax, 0x1C4),
        Load(R::Eax, 0x75ADB4),
        ToEdi(R::Eax, 0x1C8),
        Load(R::Eax, 0x75ADB8),
        ToEdi(R::Eax, 0x1CC),
        Load(R::Eax, 0x75ADBC),
        ToEdi(R::Eax, 0x1D0),
        Load(R::Eax, 0x75ADC0),
        ToEdi(R::Eax, 0x1D4),
        Load(R::Eax, 0x75ADC4),
        ToEdi(R::Eax, 0x1D8),
        Load(R::Eax, 0x75ADC8),
        ToEdi(R::Eax, 0x1DC),
        Load(R::Eax, 0x75ADCC),
        ToEdi(R::Eax, 0x1E0),
        Load(R::Eax, 0x75ADD0),
        ToEdi(R::Eax, 0x1E4),
        Load(R::Eax, 0x75ADD4),
        ToEdi(R::Eax, 0x1E8),
        Load(R::Eax, 0x75ADD8),
        ToEdi(R::Eax, 0x1EC),
        Load(R::Eax, 0x75ADDC),
        ToEdi(R::Eax, 0x1F0),
        Load(R::Eax, 0x75ADE0),
        ToEdi(R::Eax, 0x1F4),
        Load(R::Eax, 0x75ADE4),
        ToEdi(R::Eax, 0x1F8),
        Load(R::Eax, 0x75ADE8),
        ToEdi(R::Eax, 0x1FC),
        Load(R::Eax, 0x75ADEC),
        ToEdi(R::Eax, 0x200),
        Load(R::Eax, 0x75ADF0),
        ToEdi(R::Eax, 0x204),
        Load(R::Eax, 0x75ADF4),
        ToEdi(R::Eax, 0x208),
        Load(R::Ebx, 0x75AD90),
        AddMem(R::Ebx, 0x75AD94),
        AddMem(R::Ebx, 0x75AD98),
        AddMem(R::Ebx, 0x75AD9C),
        CmpReg(R::Ebx, 0x0),
        Jz(845),
        Move(R::Eax, R::Ebx),
        CmpReg(R::Eax, 0x0),
        JzFatal(1),
        Imm(R::Edx, 0),
        Imm(R::Ebp, 0xA),
        DivEbp,
        Move(R::Ebx, R::Eax),
        Load(R::Eax, 0x75ADA0),
        MulImm(R::Eax, 0x6),
        CmpReg(R::Eax, 0x0),
        Jz(855),
        Nop,
        CmpReg(R::Eax, 0x0),
        JzFatal(1),
        Imm(R::Edx, 0),
        Imm(R::Ebp, 0xA),
        DivEbp,
        AddReg(R::Ebx, R::Eax),
        ToEdi(R::Ebx, 0xD8),
        Load(R::Eax, 0x75ADD4),
        ToEdi(R::Eax, 0xF8),
        Load(R::Eax, 0x75ADF4),
        ToEdi(R::Eax, 0x108),
        Load(R::Eax, 0x75ADE0),
        ToEdi(R::Eax, 0x100),
        Load(R::Eax, 0x75ADC0),
        ToEdi(R::Eax, 0xE8),
        Load(R::Eax, 0x75AE20),
        ToEdi(R::Eax, 0x154),
        Load(R::Eax, 0x75AE24),
        ToEdi(R::Eax, 0xC),
        CmpMem(0x75B028, 0xFFFFFFFE),
        Jz(873),
        CmpMem(0x75B028, 0xFFFFFFFF),
        Jnz(876),
        EdiPut(0x88, 0x36EE80),
        EdiPut(0xD0, 0x6),
        Jmp(890),
        Load(R::Eax, 0x75B01C),
        ToEdi(R::Eax, 0x88),
        Load(R::Eax, 0x75B020),
        ToEdi(R::Eax, 0x27C),
        Load(R::Eax, 0x75B024),
        RoundEaxToEbx,
        MulImm(R::Ebx, 0xA),
        ToEdi(R::Ebx, 0x94),
        Pusha,
        Imm(R::Ecx, 0x40),
        Lea(R::Esi, 0x75B02C),
        LeaEdi(R::Edi, 0x20C),
        RepMovsb,
        Popa,
        Load(R::Eax, 0x7587E8),
        CmpReg(R::Eax, 0x1),
        Jz(896),
        Call(0x442AF6),
        Call(0x474E04),
        Call(0x442AF6),
        Call(0x407F95),
        Load(R::Edx, 0x75ACE4),
        Store(R::Edx, 0x7554C8),
        Call(0x40F780),
        Popa,
        Return,
    ];
}

#[cfg(test)]
mod tests {
    use super::table::{Op, R, SINGLE_RALLY};
    use super::*;
    use crate::errors::BadInput;
    use crate::framework::Regs;

    /// The tests share the game's memory.
    /// The size of the stand-in for the game's image: every variable of the function is below.
    const IMAGE_LEN: usize = 0x400000;
    const RECORD_LEN: usize = 0x300;

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u32 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 >> 16) as u32
        }
        fn pick<T: Copy>(&mut self, list: &[T]) -> T {
            list[self.next() as usize % list.len()]
        }
    }

    /// The game's memory for the tests: the image's stand-in (random bytes, `BASE` at its
    /// start, for the variables that have not moved to statics) and the players' records.
    struct World {
        image: usize,
        records: Vec<Vec<u8>>,
        /// The records as regions while the world lives.
        _regions: Vec<crate::pointers::TestRegion>,
        /// Every range the table (and its calls' stand-ins) reads or writes: (address, length).
        ranges: Vec<(usize, usize)>,
    }

    fn world() -> World {
        use std::sync::atomic::Ordering::Relaxed;
        let base = crate::pointers::game_address(
            Box::leak(vec![0u8; IMAGE_LEN].into_boxed_slice()).as_mut_ptr(),
        );
        crate::framework::BASE
            .compare_exchange(0, base as usize, Relaxed, Relaxed)
            .expect("no other test sets the base");
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        for i in 0..IMAGE_LEN {
            unsafe {
                crate::pointers::host_at(base as usize)
                    .add(i)
                    .write(rng.next() as u8)
            };
        }
        let records: Vec<Vec<u8>> = (0..4).map(|_| vec![0u8; RECORD_LEN]).collect();
        let mut ranges = Vec::new();
        for op in SINGLE_RALLY.iter() {
            match *op {
                Op::Store(R::Al, a) => ranges.push((game_addr(rva(a)) as usize, 1)),
                Op::Store(_, a) | Op::Load(_, a) | Op::Put(a, _) | Op::CmpMem(a, _) => {
                    ranges.push((game_addr(rva(a)) as usize, 4))
                }
                Op::AddMem(_, a) => ranges.push((game_addr(rva(a)) as usize, 4)),
                _ => {}
            }
        }
        for (a, len) in [
            (BLOCK_FIRST.addr(), 0x20),
            (BLOCK_CURRENT.addr(), 0x20),
            (OUTCOME_FIRST.data.addr(), 0x40),
            (OUTCOME_CURRENT.data.addr(), 0x40),
            (PLAYERS.addr(), 16),
        ] {
            ranges.push((a as usize, len));
        }
        for r in &records {
            ranges.push((
                crate::pointers::game_address(r.as_ptr()) as usize,
                RECORD_LEN,
            ));
        }
        ranges.sort_unstable();
        ranges.dedup();
        let _regions = records
            .iter()
            .map(|r| crate::pointers::TestRegion::new(r))
            .collect();
        World {
            image: base as usize,
            records,
            _regions,
            ranges,
        }
    }

    impl World {
        fn image_bytes(&self) -> Vec<u8> {
            unsafe {
                std::slice::from_raw_parts(
                    crate::pointers::host_at(self.image) as *const u8,
                    IMAGE_LEN,
                )
                .to_vec()
            }
        }
        /// The bytes of every range (the variables that have moved out of the image too).
        fn range_bytes(&self) -> Vec<u8> {
            let mut out = Vec::new();
            for &(a, len) in &self.ranges {
                out.extend_from_slice(unsafe {
                    std::slice::from_raw_parts(crate::pointers::host_at(a) as *const u8, len)
                });
            }
            out
        }
        fn set_range_bytes(&self, bytes: &[u8]) {
            let mut at = 0;
            for &(a, len) in &self.ranges {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        bytes[at..].as_ptr(),
                        crate::pointers::host_at(a),
                        len,
                    )
                };
                at += len;
            }
        }
        fn set_image(&self, bytes: &[u8]) {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    crate::pointers::host_at(self.image),
                    IMAGE_LEN,
                )
            }
        }
    }

    #[derive(Clone, Debug)]
    struct Case {
        start: u32,
        kind: u32,
        mode: u32,
        count: u32,
        current: u32,
        /// The +0xD0 word of each record.
        d0: [u32; 4],
        /// Records `g_players` has no entry for (0).
        missing: [bool; 4],
        stage: u32,
        /// The entry of the track list holding the stage, or the entry the list ends at.
        entry: usize,
        found: bool,
        result: Option<u32>,
        result_2: Option<u32>,
        secs: u32,
        secs_2: u32,
        /// What the call at place 9 leaves in `SWITCH`.
        switch_after: Option<u32>,
        /// How the race leaves the sums of times: 0 random, 1 zero, 2 wrapping to zero, 3 small.
        sums: u32,
        sums_2: u32,
        extra: u32,
        extra_2: u32,
        fail_at: Option<u32>,
        seed: u64,
    }

    /// The calls' stand-ins: they log (place, the memory then), may fail, and write what the
    /// real calls would change that the function reads after.
    struct Calls<'a> {
        case: &'a Case,
        ranges: &'a [(usize, usize)],
        log: Vec<(u32, u64)>,
        rng: Rng,
    }

    fn digest(ranges: &[(usize, usize)]) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for &(a, len) in ranges {
            for i in 0..len {
                h ^= unsafe { (crate::pointers::host_at(a + i) as *const u8).read() } as u64;
                h = h.wrapping_mul(0x100_0000_01b3);
            }
        }
        h
    }

    impl Calls<'_> {
        unsafe fn call(&mut self, place: u32) -> Result<(), Refused> {
            unsafe {
                let c = self.case;
                self.log.push((place, digest(self.ranges)));
                if c.fail_at == Some(place) {
                    return Err(BadInput("injected").into());
                }
                match place {
                    1 => set_var(SUMMARY, self.rng.next()),
                    3 => STAGE.set(c.stage),
                    4 => set_var(AFTER_LOADING, self.rng.next()),
                    7 => {
                        // The race changes everything the results are copied from.
                        let (kept, kept_2) = (RESULT.get(), RESULT_2.get());
                        for a in (0x75AD00..0x75B0FC).step_by(4) {
                            set_var(a, self.rng.next());
                        }
                        for (times, extra, sums, e) in [
                            (TIMES_FIRST, EXTRA_FIRST, c.sums, c.extra),
                            (TIMES_CURRENT, EXTRA_CURRENT, c.sums_2, c.extra_2),
                        ] {
                            let v: [u32; 4] = match sums {
                                1 => [0; 4],
                                2 => [1, u32::MAX, 0, 0],
                                3 => [3, 0, 2, 1],
                                _ => [(); 4].map(|_| self.rng.next()),
                            };
                            for (a, v) in times.iter().zip(v) {
                                a.set(v);
                            }
                            extra.set(e);
                        }
                        RESULT.set(c.result.unwrap_or(kept));
                        RESULT_2.set(c.result_2.unwrap_or(kept_2));
                        OUTCOME_CURRENT.secs.set_bits(c.secs);
                        OUTCOME_FIRST.secs.set_bits(c.secs_2);
                    }
                    9 => {
                        if let Some(s) = c.switch_after {
                            set_var(SWITCH, s);
                        }
                    }
                    13 => RACE_KEPT.set(self.rng.next()),
                    _ => {}
                }
                Ok(())
            }
        }
    }

    /// Puts the case's memory in place (the same bytes for both runs).
    unsafe fn setup(w: &mut World, c: &Case) {
        unsafe {
            let mut rng = Rng(c.seed | 1);
            for &(a, len) in &w.ranges {
                for i in 0..len {
                    crate::pointers::host_at(a + i).write(rng.next() as u8);
                }
            }
            for (k, r) in w.records.iter().enumerate() {
                set_var(
                    &PLAYER_TABLE.pointers[k],
                    if c.missing[k] {
                        0
                    } else {
                        crate::pointers::game_address(r.as_ptr())
                    },
                );
                set_field(crate::pointers::game_address(r.as_ptr()), 0xD0, c.d0[k])
                    .expect("the test's memory is registered");
            }
            set_var(EVENT_MODE, c.mode);
            set_var(PLAYER_COUNT, c.count);
            set_var(CURRENT, c.current);
            // The track list: entries 0x40 apart from +0x34; the stage's number at +0x34 of
            // the entry; a dword -1 at the start of the entry the list ends at.
            let entry = |k: usize| {
                crate::records::stages_store::STAGE_TABLES.trak_pak() + 0x34 + 0x40 * k as u32
            };
            for k in 0..6 {
                let mut id = rng.next();
                if id == c.stage {
                    id ^= 1;
                }
                set_field(entry(k), 0x34, id).expect("the test's memory is registered");
                if k > 0 {
                    let mut s = rng.next();
                    if s == u32::MAX {
                        s = 0;
                    }
                    set_field(entry(k), 0, s).expect("the test's memory is registered");
                }
            }
            if c.found {
                set_field(entry(c.entry), 0x34, c.stage).expect("the test's memory is registered");
            } else {
                set_field(entry(c.entry), 0, u32::MAX).expect("the test's memory is registered");
            }
        }
    }

    type Outcome3 = (Result<(), Refused>, Vec<(u32, u64)>, Vec<u8>, Vec<u8>);

    fn after(w: &World, result: Result<(), Refused>, log: Vec<(u32, u64)>) -> Outcome3 {
        (result, log, w.image_bytes(), w.range_bytes())
    }

    /// The table, with its own registers; the stand-ins leave junk in the ones the real
    /// calls leave (eax, ebx, ecx, edx, al).
    unsafe fn run_table(w: &World, c: &Case) -> Outcome3 {
        unsafe {
            let mut calls = Calls {
                case: c,
                ranges: &w.ranges,
                log: Vec::new(),
                rng: Rng(c.seed ^ 0x55AA),
            };
            let result = if player(1).unwrap() == 0 || player(var(CURRENT)).unwrap() == 0 {
                Err(BadInput("a player's record missing").into())
            } else {
                super::table::run(
                    &SINGLE_RALLY,
                    c.start,
                    c.kind,
                    &mut |place, g: &mut Regs| {
                        let junk = 0xDEAD_0000 | place;
                        match place {
                            4 => g.eax = junk,
                            11 => {
                                [g.eax, g.ebx, g.ecx, g.edx] = [junk, junk + 1, junk + 2, junk + 3]
                            }
                            13 => g.eax = (g.eax & !0xFF) | 0x41,
                            _ => {}
                        }
                        calls.call(place)
                    },
                )
            };
            after(w, result, calls.log)
        }
    }

    unsafe fn run_new(w: &World, c: &Case) -> Outcome3 {
        unsafe {
            let mut calls = Calls {
                case: c,
                ranges: &w.ranges,
                log: Vec::new(),
                rng: Rng(c.seed ^ 0x55AA),
            };
            let result = guarded(c.start, c.kind, &mut |place| calls.call(place));
            after(w, result, calls.log)
        }
    }

    fn cases() -> Vec<Case> {
        let mut rng = Rng(0x1234_5678_9ABC_DEF1);
        let floats = [
            0.0f32.to_bits(),
            1.5f32.to_bits(),
            2.5f32.to_bits(),
            12.75f32.to_bits(),
            (-3.5f32).to_bits(),
            123456.7f32.to_bits(),
            3e9f32.to_bits(),
            0x7FC0_0000,
        ];
        let results = [None, Some(-2i32 as u32), Some(u32::MAX), Some(0), Some(7)];
        let mut out = Vec::new();
        let make = |rng: &mut Rng, kind, start, mode, count, current| {
            let found = !rng.next().is_multiple_of(8);
            Case {
                start,
                kind,
                mode,
                count,
                current,
                d0: [(); 4].map(|_| rng.pick(&[1, 1, 0, 2])),
                missing: [false, false, false, current == 3],
                stage: rng.next(),
                entry: if found {
                    rng.pick(&[0, 1, 2, 4])
                } else {
                    rng.pick(&[1, 2, 3])
                },
                found,
                result: rng.pick(&results),
                result_2: rng.pick(&results),
                secs: rng.pick(&floats),
                secs_2: rng.pick(&floats),
                switch_after: rng.pick(&[None, Some(0), Some(1)]),
                sums: rng.pick(&[0, 1, 2, 3]),
                sums_2: rng.pick(&[0, 1, 2, 3]),
                extra: {
                    let r = rng.next();
                    rng.pick(&[0, 1, 2, 0x2AAA_AAAB, r])
                },
                extra_2: {
                    let r = rng.next();
                    rng.pick(&[0, 1, 2, 0x2AAA_AAAB, r])
                },
                fail_at: None,
                seed: rng.next() as u64 | 1 << 40,
            }
        };
        for kind in [3, 1, 7] {
            for start in [1, 0, 2] {
                for mode in [5, 4, 0xF, 0xA, 9, 0x12] {
                    for count in [1, 2] {
                        for current in [0, 1, 2, 3] {
                            for _ in 0..3 {
                                out.push(make(&mut rng, kind, start, mode, count, current));
                            }
                        }
                    }
                }
            }
        }
        // Each call failing, in both families of modes, with a track found.
        for place in 1..=14 {
            for mode in [5, 9] {
                for start in [1, 0] {
                    let mut c = make(&mut rng, 3, start, mode, 2, 1);
                    c.found = true;
                    c.entry = 1;
                    c.d0 = [1; 4];
                    c.fail_at = Some(place);
                    out.push(c);
                }
            }
        }
        // A player's record missing: player 1, and the current.
        for mode in [5, 9] {
            let mut c = make(&mut rng, 3, 1, mode, 2, 1);
            c.missing = [false, true, false, false];
            out.push(c);
        }
        out
    }

    /// The table and the plain Rust version, on the same memory and calls, agree on the
    /// result, the order of the calls with the memory at each, and every byte left.
    #[test]
    fn plain_rust_equals_the_table() {
        let _g = crate::store::test_lock();
        let mut w = world();
        // The game's variables as they were, put back at the end (other tests read them).
        let first = crate::moved::snapshot();
        // A first check that the stand-in memory is there.
        RACE_INFO.set(0x1234_5678);
        assert_eq!(RACE_INFO.get(), 0x1234_5678);
        let (mut done, mut fatal, mut failed, mut missing, mut engine, mut early, mut results) =
            (0, 0, 0, 0, 0, 0, 0);
        let cases = cases();
        for (n, c) in cases.iter().enumerate() {
            unsafe {
                setup(&mut w, c);
                let init_image = w.image_bytes();
                let init_ranges = w.range_bytes();
                let reference = run_table(&w, c);
                // Back to the start: the image, and every range (the records, the variables
                // that have left the image).
                w.set_image(&init_image);
                w.set_range_bytes(&init_ranges);
                let ours = run_new(&w, c);
                assert_eq!(ours.0, reference.0, "case {n}: result {c:?}");
                assert_eq!(ours.1, reference.1, "case {n}: calls {c:?}");
                assert!(ours.2 == reference.2, "case {n}: image {c:?}");
                assert!(ours.3 == reference.3, "case {n}: ranges {c:?}");
                match reference.0 {
                    Ok(()) => done += 1,
                    Err(Refused::Fatal(_)) => fatal += 1,
                    Err(Refused::BadInput(BadInput("injected"))) => failed += 1,
                    Err(_) => missing += 1,
                }
                let places: Vec<u32> = reference.1.iter().map(|&(p, _)| p).collect();
                engine += places.contains(&11) as u32;
                early += (places.last() == Some(&14) && !places.contains(&5)) as u32;
                results += (places.contains(&7) && c.result == Some(u32::MAX)) as u32;
            }
        }
        assert!(
            done > 100
                && fatal > 10
                && failed > 20
                && missing > 0
                && engine > 20
                && early > 5
                && results > 10,
            "coverage: {done} {fatal} {failed} {missing} {engine} {early} {results}"
        );
        crate::moved::restore(&first);
    }
}
