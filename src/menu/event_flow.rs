//! What the event flows share (IDA, 2026-10-19): the championship drivers
//! (`menu::run_championship`, `menu::run_a8_championship`), their starters, the single
//! rally's and the time trial's multi-player events. Each of them is a sequence of calls
//! and of tests of the game's variables, so their Rust versions take every callee through
//! a `Call` (the callee by its address as IDA shows it, with the registers the original
//! has there), and `game_call` is that in the game: the Rust versions of the callees, the
//! screens through `menu::screen_loop`, and the race, the repairs' clock, the sound commands
//! and the frame through their hooks (the platform's or long runners: their hooks choose).
//!
//! For the self-tests, `test_flow`: a copy of the original whose every call goes to a
//! stand-in that notes the callee with eax .. ebp and gives a scripted answer (variables
//! written, eax, al, edi, the carry and zero flags, a record for `records::find_record`),
//! against the Rust version given the same answers. The game's variables are used as they
//! are and put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, SelfTest, call_game, game_addr, game_read, game_write, rva};
use crate::menu::screen_loop::run_screen;
use crate::service::harness::{Region, at, capture, restore};
use std::collections::VecDeque;
use std::sync::Mutex;

/// A callee by its address as IDA shows it, with the registers it gets and leaves.
pub type Call<'a> = &'a mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>;

/// A game variable (its address as IDA shows it).
pub fn get(v: usize) -> u32 {
    unsafe { game_read(rva(v)) }
}

pub fn set(v: usize, x: u32) {
    unsafe { game_write(rva(v), x) };
}

pub fn inc(v: usize) {
    set(v, get(v).wrapping_add(1));
}

pub fn dword(p: u32) -> u32 {
    unsafe { (p as *const u32).read_unaligned() }
}

pub fn put(p: u32, v: u32) {
    unsafe { (p as *mut u32).write_unaligned(v) };
}

// ---- the variables ----

pub const ANSWER: usize = 0x75524C; // g_yesNoAnswer
pub const PLAYERS: usize = 0x75647C; // g_players
pub const CURRENT: usize = 0x75598C; // dword_75598C: the current player
pub const PLAYER_COUNT: usize = 0x755984; // dword_755984
pub const EVENT_MODE: usize = 0x755988; // dword_755988 (9 .. 0x18, `menu::run_event_races`)
pub const EVENT_TYPE: usize = 0x755954; // dword_755954: 3 time trial, 4, 5 the championships
pub const MODE: usize = 0x7556D0; // g_eventMode: 2, 3, 0x13
pub const CAR_LEFT: usize = 0x758806; // dword_758806: the car choice was left
pub const LEAVE: usize = 0x75555D; // dword_75555D
pub const LEAVE_561: usize = 0x755561; // dword_755561
pub const ENDED_565: usize = 0x755565; // dword_755565
pub const ENDED_569: usize = 0x755569; // dword_755569
pub const DONE: usize = 0x755980; // dword_755980: 1 when an event is over
pub const LEFT_A3C: usize = 0x755A3C; // dword_755A3C
pub const CONTINUING: usize = 0x755970; // dword_755970: 1 when a saved game goes on
pub const INDEX: usize = 0x755958; // index
pub const SECTION: usize = 0x75595C; // section
pub const RECORD_TABLE: usize = 0x735576; // g_recordTable
pub const CHEATS: usize = 0x75870C; // g_cheats
pub const RACE_RESULT: usize = 0x75B028; // dword_75B028: -1 no result, -2 left
pub const RACE_RESULT_2: usize = 0x75B098; // dword_75B098: the second car's
pub const ROUND: usize = 0x758714; // dword_758714: the rounds of a multi-player event
pub const QUITTING: usize = 0x51837C; // g_quitting

/// The car lists: `off_759CAE` (the championship's), `off_759D16` (the A8 championship's),
/// `off_759DCA` / `off_75A042` (by `dword_7554D8`), `off_75A412` (with cheat bit 1).
pub const CARS_CHAMPIONSHIP: usize = 0x759CAE;
pub const CARS_A8: usize = 0x759D16;
pub const CARS_FIRST: usize = 0x759DCA;
pub const CARS_SECOND: usize = 0x75A042;
pub const CARS_CHEAT: usize = 0x75A412;
pub const CAR_LIST_KIND: usize = 0x7554D8; // dword_7554D8: 1 picks the first list

/// Player `n`'s record (`g_players`).
pub fn player(n: u32) -> u32 {
    get(PLAYERS + 4 * n as usize)
}

/// Refuses a missing one of the first `n` players (the originals read through them).
pub fn players_there(n: u32) -> Result<(), BadInput> {
    if (0..n).any(|k| player(k) == 0) {
        return Err(BadInput("a player missing from g_players"));
    }
    Ok(())
}

pub fn yes() -> bool {
    get(ANSWER) == 1
}

/// The current player's record, refused past the four or missing.
pub fn current_player() -> Result<u32, BadInput> {
    let n = get(CURRENT);
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past the four or missing"));
    }
    Ok(player(n))
}

/// The stage chosen: `off_735512[dword_7558C8][dword_7558D0]` + `dword_7558CC` x 0xA4.
pub const STAGE_KIND: usize = 0x7558C8; // dword_7558C8
pub const STAGE_SUB: usize = 0x7558D0; // dword_7558D0
pub const STAGE_NUMBER: usize = 0x7558CC; // dword_7558CC
const STAGE_TABLES: usize = 0x735512; // off_735512: four pointers
const KINDS: u32 = 4;
pub const STAGE_RECORD: u32 = 0xA4;
/// The stages' best times: `dword_7556E4` (kind 1) or `dword_7556E8`, 0x618 a stage (its
/// +0x3C), ten entries of 0x9C each.
pub const TIMES_ONE: usize = 0x7556E4; // dword_7556E4
pub const TIMES_OTHER: usize = 0x7556E8; // dword_7556E8
pub const TIMES_LEN: u32 = 0x618;
pub const TIME_ENTRY: u32 = 0x9C;

/// The chosen stage's record; refused for a kind past the table's four or a null pointer
/// on the way (the original reads through it).
pub fn stage_record() -> Result<u32, BadInput> {
    let kind = get(STAGE_KIND);
    if kind >= KINDS {
        return Err(BadInput("a stage kind past off_735512's four"));
    }
    let rallies = dword(game_addr(rva(STAGE_TABLES)) as u32 + 4 * kind);
    let stages = if rallies == 0 {
        0
    } else {
        dword(rallies.wrapping_add(get(STAGE_SUB).wrapping_mul(4)))
    };
    if stages == 0 {
        return Err(BadInput("a null pointer to the chosen stage's records"));
    }
    Ok(stages.wrapping_add(get(STAGE_NUMBER).wrapping_mul(STAGE_RECORD)))
}

/// The chosen stage's best times (ten entries); refused as `stage_record`, or with no
/// table.
pub fn stage_times() -> Result<u32, BadInput> {
    let record = stage_record()?;
    let base = get(if get(STAGE_KIND) == 1 {
        TIMES_ONE
    } else {
        TIMES_OTHER
    });
    if base == 0 {
        return Err(BadInput("no best times table"));
    }
    Ok(base.wrapping_add(dword(record + 0x3C).wrapping_mul(TIMES_LEN)))
}

/// For the time trial's tests: the chosen stage's record and best times, as regions.
pub fn stage_regions() -> Option<Vec<Region>> {
    Some(vec![
        (stage_record().ok()?, STAGE_RECORD as usize),
        (stage_times().ok()?, TIMES_LEN as usize),
    ])
}

/// For the time trial's tests: the stage choices tried (kind, sub, number): the game's,
/// and two more where their tables are there.
pub fn stage_choices() -> Vec<[u32; 3]> {
    let game = [get(STAGE_KIND), get(STAGE_SUB), get(STAGE_NUMBER)];
    let mut out = Vec::new();
    for c in [game, [0, 0, 1], [1, 2, 3]] {
        set(STAGE_KIND, c[0]);
        set(STAGE_SUB, c[1]);
        set(STAGE_NUMBER, c[2]);
        if stage_regions().is_some() && !out.contains(&c) {
            out.push(c);
        }
    }
    set(STAGE_KIND, game[0]);
    set(STAGE_SUB, game[1]);
    set(STAGE_NUMBER, game[2]);
    out
}

pub fn choose_stage(c: [u32; 3]) {
    set(STAGE_KIND, c[0]);
    set(STAGE_SUB, c[1]);
    set(STAGE_NUMBER, c[2]);
}

// ---- the callees ----

pub const BEGIN_EVENT: usize = 0x472BAA;
pub const SHUFFLE_DRIVERS: usize = 0x470AB9;
pub const CLEAR_RESULT_SLOTS: usize = 0x470DDA;
pub const RESET_DRIVERS: usize = 0x470399;
pub const RESET_DRIVER_CONDITION: usize = 0x47E7BC;
pub const CLEAR_PLAYERS_284: usize = 0x4A9679;
pub const TAKE_PLAYER1_UNITS: usize = 0x467EA8;
pub const RESET_CARS_750724: usize = 0x46FC52;
pub const CHAMPIONSHIP_WELCOME: usize = 0x47F314;
pub const A8_WELCOME: usize = 0x47F3C7;
pub const STOP_STREAM: usize = 0x40F757;
pub const START_STREAM: usize = 0x40F743;
pub const WAIT_FOR_COMMANDS: usize = 0x409C1E;
pub const FIND_RECORD: usize = 0x47255D;
pub const WELCOME_LOOP: usize = 0x4721F0;
pub const STANDINGS_B_FROM_ALL: usize = 0x4A44A0;
pub const SORT_B_BY_FIGURE: usize = 0x4A467A;
pub const AWARD_STAGE_POINTS: usize = 0x4A47EA;
pub const SORT_B_BY_POINTS: usize = 0x4A5B07;
pub const CLASS_CHAMPIONSHIP_PASS: usize = 0x4A4A1C;
pub const AWARD_OVERALL_POINTS: usize = 0x4A4933;
pub const CLEAR_STANDINGS_B: usize = 0x4A5A2F;
pub const SORT_B_BY_SCORE: usize = 0x4A5C77;
pub const NUMBER_ALL_B: usize = 0x4A5A98;
pub const SHOW_CHAMPIONSHIP_STANDINGS: usize = 0x4A5A47;
pub const SECTION_FINISH: usize = 0x4A9439;
pub const REVIEW_480D46: usize = 0x480D46;
pub const RALLY_STAGES_LOOP: usize = 0x472596;
pub const RESET_PLAYER_TIMES: usize = 0x4813D6;
pub const NEXT_STAGE_LOOP: usize = 0x473B21;
pub const SERVICE_PASS: usize = 0x4A15E3;
pub const WALK_SECTION_RECORDS: usize = 0x471F0D;
pub const STANDINGS_A_FROM_ALL: usize = 0x4A2B75;
pub const CLASS_RALLY_RESULTS_PASS: usize = 0x4A32A5;
pub const CLEAR_STANDINGS_A: usize = 0x4A5A14;
pub const NUMBER_ALL_A: usize = 0x4A2090;
pub const SHOW_OVERALL_RALLY_RESULTS: usize = 0x4A3BA2;
pub const RUN_EVENT_RACES: usize = 0x473126;
pub const ADD_PLAYER_TIMES: usize = 0x4A25AD;
pub const CLASS_STAGE_RESULTS_PASS: usize = 0x4A1EE6;
pub const SHOW_OVERALL_STAGE_RESULTS: usize = 0x4A1FE5;
pub const NEXT_RECORD_KIND: usize = 0x470D08;
pub const KEEP_RECORD_BEST: usize = 0x4A3E8B;
pub const RECORD_ENTRIES_PASS: usize = 0x4A3BE0;
pub const BEST_TIMES_LOOP: usize = 0x4A3EF3;
pub const INSPECTION_PASS: usize = 0x4A8556;
pub const REVIEW_47D5A5: usize = 0x47D5A5;
pub const REVIEW_48108E: usize = 0x48108E;
pub const CHAMPIONSHIP_FINISH: usize = 0x4A14CB;
pub const CLEAR_SCORE_MARKS: usize = 0x4A919A;
pub const BEST_CLASS_SCORES: usize = 0x4A8608;
pub const BEST_OVERALL_SCORES: usize = 0x4A91BF;
pub const REVIEW_481429: usize = 0x481429;
pub const SHOW_IN_GAME_OPTIONS: usize = 0x46FBE3;
pub const HIDE_IN_GAME_OPTIONS: usize = 0x46FB63;
pub const RESET_SINGLE_EVENT: usize = 0x4AAA2F;
pub const REPAIR_CAR: usize = 0x486F63;
pub const SELECT_CAR_LOOP: usize = 0x47AC0C;
pub const TAKE_SHOWN_CAR: usize = 0x47B29D;
pub const RUN_CHAMPIONSHIP: usize = 0x4A0E60;
pub const RUN_A8_CHAMPIONSHIP: usize = 0x4AA4D7;
pub const CLEAR_EVENT_STATE: usize = 0x486C2C;
pub const READY_PLAYERS: usize = 0x471F7E;
pub const SELECT_STAGE_LOOP: usize = 0x482099;
pub const RACE: usize = 0x4843EC; // race::single_rally_race
pub const RESULTS_48584B: usize = 0x48584B;
pub const REVIEW_485F5E: usize = 0x485F5E;
pub const RELOAD_FRONT_END: usize = 0x48140D;
pub const RESULTS_482E9B: usize = 0x482E9B;
pub const SCREEN_483163: usize = 0x483163;
pub const CHEAT_CAR_LIST: usize = 0x480D35;
pub const TIME_TRIAL_SELECT_STAGE: usize = 0x48BFCB;
pub const TIME_TRIAL_SERVICE: usize = 0x48E093;
pub const STAGE_RESULT_LOOP: usize = 0x48558D;
pub const KEEP_STAGE_BEST: usize = 0x48E02D;
pub const TIME_TRIAL_QUALIFIES: usize = 0x48C7CE;
pub const CLEAR_TIME_TRIAL_MARKS: usize = 0x48C85C;
pub const ENTER_TIME_TRIAL_BEST: usize = 0x48C8B6;
pub const RANK_TIME_TRIAL_BEST: usize = 0x48CC51;
pub const SEALED_PLAYER_TIME: usize = 0x4A3D38;
pub const TIME_TRIAL_BEST_TIMES_LOOP: usize = 0x48CE48;
pub const TIME_TRIAL_RESULTS_LOOP: usize = 0x48DB7D;
pub const INSPECTION_SCREEN: usize = 0x4A1BBF;
pub const SLEEP: usize = 0x41100C;
pub const FADE_IN: usize = 0x4A1CFA;
pub const DRIVER_STATS: usize = 0x4A73D8;
pub const SERVICE_AREA_LOOP: usize = 0x47284A;
pub const APPLY_REPAIRS: usize = 0x46C14D;
pub const SET_PLAYER_ENTRY_KIND: usize = 0x4A1922;
pub const INSPECTION_FAULTS: usize = 0x4A1733;
pub const INSPECTION_AREA_LOOP: usize = 0x47DBF3;
pub const LOAD_PICTURE_WITH_OVERLAY: usize = 0x4118D1;
pub const CONVERT_PICTURE: usize = 0x406126;
pub const TAKE_KEY: usize = 0x407F95;
pub const RUN_FRAME: usize = 0x405A71;
pub const WAIT_OR_KEY: usize = 0x407F69;
pub const STREAM_PLAYING: usize = 0x40C4D5;
pub const CONFIRM_LEAVING: usize = 0x46FA90;
pub const PICTURE_FRAME: usize = 0x46F3FA; // menu::picture_frame, handed to the frame

/// In the game: a callee by its address (see the header). An address not listed is a
/// slip of a Rust version's and refused.
pub fn game_call(callee: usize, r: &mut Regs) -> Result<(), Refused> {
    use crate::menu as m;
    unsafe {
        match callee {
            BEGIN_EVENT => m::begin_event::begin_event()?,
            SHUFFLE_DRIVERS => *r = keep_esp(r, m::shuffle_drivers::shuffle_drivers(r)?),
            CLEAR_RESULT_SLOTS => *r = keep_esp(r, m::clear_result_slots::clear_result_slots(r)?),
            RESET_DRIVERS => *r = keep_esp(r, m::reset_drivers::reset_drivers(r)?),
            RESET_DRIVER_CONDITION => {
                *r = keep_esp(r, m::reset_driver_condition::reset_driver_condition(r)?)
            }
            CLEAR_PLAYERS_284 => m::clear_players_284::clear_players_284()?,
            TAKE_PLAYER1_UNITS => m::take_player1_units::take_player1_units()?,
            RESET_CARS_750724 => *r = keep_esp(r, m::reset_cars_750724::reset_cars_750724(r)?),
            CHAMPIONSHIP_WELCOME => {
                *r = keep_esp(
                    r,
                    m::championship_welcome::championship_welcome(r, &mut game_call)?,
                )
            }
            A8_WELCOME => *r = keep_esp(r, m::a8_welcome::a8_welcome(r, &mut game_call)?),
            FIND_RECORD => {
                use crate::records::find_record::{find_record_in, has_section};
                // The original has no end-of-table check: a section past the table's is
                // refused.
                if !has_section(r.edi, r.ebx, 600) {
                    return Err(BadInput("a record section past the table").into());
                }
                r.edi = find_record_in(r.edi, r.eax as i32, r.ebx)
            }
            STANDINGS_B_FROM_ALL => {
                *r = keep_esp(r, m::standings_b_from_all::standings_b_from_all(r)?)
            }
            SORT_B_BY_FIGURE => {
                *r = keep_esp(
                    r,
                    m::sort_standings_b_by_figure::sort_standings_b_by_figure(r)?,
                )
            }
            AWARD_STAGE_POINTS => *r = keep_esp(r, m::award_stage_points::award_stage_points(r)?),
            SORT_B_BY_POINTS => {
                *r = keep_esp(
                    r,
                    m::sort_standings_b_by_points::sort_standings_b_by_points(r)?,
                )
            }
            CLASS_CHAMPIONSHIP_PASS => {
                use m::class_championship_pass as f;
                *r = keep_esp(r, f::class_championship_pass(r, &mut f::game_part)?)
            }
            AWARD_OVERALL_POINTS => {
                *r = keep_esp(r, m::award_overall_points::award_overall_points(r)?)
            }
            CLEAR_STANDINGS_B => *r = keep_esp(r, m::clear_standings_b::clear_standings_b(r)?),
            SORT_B_BY_SCORE => {
                *r = keep_esp(
                    r,
                    m::sort_standings_b_by_score::sort_standings_b_by_score(r)?,
                )
            }
            NUMBER_ALL_B => *r = keep_esp(r, m::number_all_b::number_all_b(r)?),
            SHOW_CHAMPIONSHIP_STANDINGS => {
                use m::show_championship_standings as f;
                *r = keep_esp(r, f::show_championship_standings(r, &mut f::game_part)?)
            }
            SECTION_FINISH => {
                *r = keep_esp(
                    r,
                    m::section_finish::section_finish(r, &mut m::finish_calls::game_part)?,
                )
            }
            CHAMPIONSHIP_FINISH => {
                *r = keep_esp(
                    r,
                    m::championship_finish::championship_finish(
                        r,
                        &mut m::finish_calls::game_part,
                    )?,
                )
            }
            RESET_PLAYER_TIMES => *r = keep_esp(r, m::reset_player_times::reset_player_times(r)?),
            SERVICE_PASS => {
                use m::service_pass as f;
                *r = keep_esp(r, f::service_pass(r, &mut f::game_part)?)
            }
            WALK_SECTION_RECORDS => {
                *r = keep_esp(r, m::walk_section_records::walk_section_records(r)?)
            }
            STANDINGS_A_FROM_ALL => {
                *r = keep_esp(r, m::standings_a_from_all::standings_a_from_all(r)?)
            }
            CLASS_RALLY_RESULTS_PASS => {
                use m::class_rally_results_pass as f;
                *r = keep_esp(r, f::class_rally_results_pass(r, &mut f::game_part)?)
            }
            CLEAR_STANDINGS_A => *r = keep_esp(r, m::clear_standings_a::clear_standings_a(r)?),
            NUMBER_ALL_A => *r = keep_esp(r, m::number_all_a::number_all_a(r)?),
            SHOW_OVERALL_RALLY_RESULTS => {
                use m::show_overall_rally_results as f;
                *r = keep_esp(r, f::show_overall_rally_results(r, &mut f::game_part)?)
            }
            RUN_EVENT_RACES => {
                use m::run_event_races as f;
                *r = keep_esp(r, f::run_event_races(r, &mut f::game_part)?)
            }
            ADD_PLAYER_TIMES => *r = keep_esp(r, m::add_player_times::add_player_times(r)?),
            CLASS_STAGE_RESULTS_PASS => {
                use m::class_stage_results_pass as f;
                *r = keep_esp(r, f::class_stage_results_pass(r, &mut f::game_part)?)
            }
            SHOW_OVERALL_STAGE_RESULTS => {
                use m::show_overall_stage_results as f;
                *r = keep_esp(r, f::show_overall_stage_results(r, &mut f::game_part)?)
            }
            NEXT_RECORD_KIND => *r = keep_esp(r, m::next_record_kind::next_record_kind(r)?),
            KEEP_RECORD_BEST => *r = keep_esp(r, m::keep_record_best::keep_record_best(r)?),
            RECORD_ENTRIES_PASS => {
                *r = keep_esp(r, m::record_entries_pass::record_entries_pass(r)?)
            }
            INSPECTION_PASS => {
                use m::inspection_pass as f;
                *r = keep_esp(r, f::inspection_pass(r, &mut f::game_part)?)
            }
            CLEAR_SCORE_MARKS => *r = keep_esp(r, m::clear_score_marks::clear_score_marks(r)?),
            BEST_CLASS_SCORES => {
                use m::best_class_scores as f;
                *r = keep_esp(r, f::best_class_scores(r, &mut f::game_part)?)
            }
            BEST_OVERALL_SCORES => {
                use m::best_overall_scores as f;
                *r = keep_esp(r, f::best_overall_scores(r, &mut f::game_part)?)
            }
            SHOW_IN_GAME_OPTIONS => m::in_game_options_items::show_in_game_options(),
            HIDE_IN_GAME_OPTIONS => m::in_game_options_items::hide_in_game_options(r.eax),
            RESET_SINGLE_EVENT => *r = keep_esp(r, m::reset_single_event::reset_single_event(r)?),
            REPAIR_CAR => crate::service::repair_parts::repair_car()?,
            TAKE_SHOWN_CAR => m::take_shown_car::take_shown_car(r.esi)?,
            RUN_CHAMPIONSHIP => {
                *r = keep_esp(r, m::run_championship::run_championship(r, &mut game_call)?)
            }
            RUN_A8_CHAMPIONSHIP => {
                *r = keep_esp(
                    r,
                    m::run_a8_championship::run_a8_championship(r, &mut game_call)?,
                )
            }
            CLEAR_EVENT_STATE => m::clear_event_state::clear_event_state(),
            READY_PLAYERS => m::ready_players::ready_players()?,
            CHEAT_CAR_LIST => m::cheat_car_list::cheat_car_list(r),
            TIME_TRIAL_SERVICE => {
                *r = keep_esp(
                    r,
                    m::time_trial_service::time_trial_service(r, &mut game_call)?,
                )
            }
            KEEP_STAGE_BEST => m::keep_stage_best::keep_stage_best()?,
            TIME_TRIAL_QUALIFIES => m::time_trial_qualifies::time_trial_qualifies()?,
            CLEAR_TIME_TRIAL_MARKS => m::clear_time_trial_marks::clear_time_trial_marks()?,
            ENTER_TIME_TRIAL_BEST => m::enter_time_trial_best::enter_time_trial_best(r)?,
            RANK_TIME_TRIAL_BEST => m::rank_time_trial_best::rank_time_trial_best()?,
            SEALED_PLAYER_TIME => *r = keep_esp(r, m::sealed_player_time::sealed_player_time(r)?),
            INSPECTION_SCREEN => m::inspection_pass::game_part(5, r)?,
            FADE_IN => m::inspection_pass::game_part(3, r)?,
            SLEEP => crate::system::sleep::sleep(r.eax),
            DRIVER_STATS => *r = keep_esp(r, m::driver_stats::driver_stats(r)?),
            SET_PLAYER_ENTRY_KIND => {
                *r = keep_esp(r, m::set_player_entry_kind::set_player_entry_kind(r)?)
            }
            INSPECTION_FAULTS => *r = keep_esp(r, m::inspection_faults::inspection_faults(r)?),
            LOAD_PICTURE_WITH_OVERLAY => {
                let carry = crate::gfx::load_picture_with_overlay::load_picture_with_overlay(
                    r.eax as *mut u8,
                    r.ebx as *mut u8,
                )?;
                r.set_carry(carry);
            }
            CONVERT_PICTURE => crate::gfx::convert_picture::convert_picture_now()?,
            TAKE_KEY => {
                let key = crate::input::take_key::take_key();
                r.eax = (r.eax & !0xFF) | u32::from(key);
                r.set_zero(key == 0);
            }
            WAIT_OR_KEY => {
                use crate::system::wait_or_key as f;
                *r = keep_esp(r, f::wait_or_key(r, &mut f::game_part)?)
            }
            STREAM_PLAYING => {
                let (zero, carry) = crate::sound::stream_playing::stream_playing();
                r.set_zero(zero);
                r.set_carry(carry);
            }
            CONFIRM_LEAVING => m::confirm_leaving::confirm_leaving(r, &mut |r| {
                run_screen(&m::leave_question_loop::SCREEN, r)
            }),
            // The screens.
            WELCOME_LOOP => run_screen(&m::welcome_loop::SCREEN, r),
            REVIEW_480D46 => run_screen(&m::review_options_480d46_loop::SCREEN, r),
            RALLY_STAGES_LOOP => run_screen(&m::rally_stages_loop::SCREEN, r),
            NEXT_STAGE_LOOP => run_screen(&m::next_stage_loop::SCREEN, r),
            BEST_TIMES_LOOP => run_screen(&m::best_times_loop::SCREEN, r),
            REVIEW_47D5A5 => run_screen(&m::review_options_47d5a5_loop::SCREEN, r),
            REVIEW_48108E => run_screen(&m::review_options_48108e_loop::SCREEN, r),
            REVIEW_481429 => run_screen(&m::review_options_481429_loop::SCREEN, r),
            SELECT_CAR_LOOP => run_screen(&m::select_car_loop::SCREEN, r),
            SELECT_STAGE_LOOP => run_screen(&m::select_stage_loop::SCREEN, r),
            RESULTS_48584B => run_screen(&m::results_48584b_loop::SCREEN, r),
            REVIEW_485F5E => run_screen(&m::review_options_485f5e_loop::SCREEN, r),
            RESULTS_482E9B => run_screen(&m::results_482e9b_loop::SCREEN, r),
            SCREEN_483163 => run_screen(&m::screen_483163_loop::SCREEN, r),
            TIME_TRIAL_SELECT_STAGE => run_screen(&m::time_trial_select_stage_loop::SCREEN, r),
            STAGE_RESULT_LOOP => run_screen(&m::stage_result_loop::SCREEN, r),
            TIME_TRIAL_BEST_TIMES_LOOP => run_screen(&m::time_trial_best_times_loop::SCREEN, r),
            TIME_TRIAL_RESULTS_LOOP => run_screen(&m::time_trial_results_loop::SCREEN, r),
            INSPECTION_AREA_LOOP => run_screen(&m::inspection_area_loop::SCREEN, r),
            SERVICE_AREA_LOOP => run_screen(&crate::service::service_area_loop::SCREEN, r),
            // Through their hooks: the race and the front end's set-up after it, the
            // repairs (their clock), the sound commands (the audio thread's queue), the
            // frame (the platform's).
            RACE | RELOAD_FRONT_END | APPLY_REPAIRS | STOP_STREAM | START_STREAM
            | WAIT_FOR_COMMANDS | RUN_FRAME => call_game(r, game_addr(rva(callee))),
            _ => return Err(BadInput("a callee the event flows do not know").into()),
        }
    }
    Ok(())
}

/// A Rust version's registers, the stack pointer kept.
fn keep_esp(r: &Regs, left: Regs) -> Regs {
    Regs { esp: r.esp, ..left }
}

// ---- for the self-tests ----

/// What a stand-in does: dwords written (addresses as IDA shows them), eax, al, edi, the
/// carry and zero flags, and a record for `records::find_record` (edi points at it).
#[derive(Clone, Default)]
pub struct Answer {
    pub writes: Vec<(usize, u32)>,
    pub eax: Option<u32>,
    pub al: Option<u8>,
    pub edi: Option<u32>,
    pub carry: Option<bool>,
    pub zero: Option<bool>,
    pub record: Option<[u32; RECORD_DWORDS]>,
    /// Dwords written in players' records: (player, offset, value).
    pub player_writes: Vec<(u32, u32, u32)>,
}

pub const RECORD_DWORDS: usize = 0x10;

impl Answer {
    pub fn none() -> Self {
        Self::default()
    }
    pub fn write(at: usize, v: u32) -> Self {
        Self::default().and(at, v)
    }
    /// A "yes" to leaving.
    pub fn yes() -> Self {
        Self::write(ANSWER, 1)
    }
    pub fn and(mut self, at: usize, v: u32) -> Self {
        self.writes.push((at, v));
        self
    }
    pub fn and_player(mut self, n: u32, offset: u32, v: u32) -> Self {
        self.player_writes.push((n, offset, v));
        self
    }
    pub fn eax(v: u32) -> Self {
        Self {
            eax: Some(v),
            ..Self::default()
        }
    }
    /// A key in al, zero when it is 0 (`input::take_key`, `system::wait_or_key`).
    pub fn key(k: u8) -> Self {
        Self {
            al: Some(k),
            zero: Some(k == 0),
            ..Self::default()
        }
    }
    pub fn zero(z: bool) -> Self {
        Self {
            zero: Some(z),
            ..Self::default()
        }
    }
    pub fn carry(c: bool) -> Self {
        Self {
            carry: Some(c),
            ..Self::default()
        }
    }
    /// A record of the kind given (+0), its +0xC, +0x14 and +0x38.
    pub fn record(kind: u32, c: u32, x14: u32, x38: u32) -> Self {
        let mut rec = [0u32; RECORD_DWORDS];
        for (n, d) in rec.iter_mut().enumerate() {
            *d = 0x1100 + n as u32;
        }
        rec[0] = kind;
        rec[3] = c;
        rec[5] = x14;
        rec[0xE] = x38;
        Self {
            record: Some(rec),
            ..Self::default()
        }
    }
}

/// A run's answers by callee: each call takes the next, then `Answer::none()`.
pub type Script = Vec<(usize, Vec<Answer>)>;

/// More calls than this, and every call answers so that the flow ends.
const LIMIT: usize = 1500;
const RECORD_SLOTS: usize = 256;

struct State {
    callees: Vec<usize>,
    script: Vec<(usize, VecDeque<Answer>)>,
    notes: Vec<[u32; 8]>,
    records: u32,
    next_record: usize,
}

static STATE: Mutex<State> = Mutex::new(State {
    callees: Vec::new(),
    script: Vec::new(),
    notes: Vec::new(),
    records: 0,
    next_record: 0,
});

fn begin(script: &Script) {
    let mut s = STATE.lock().unwrap();
    s.script = script
        .iter()
        .map(|(c, a)| (*c, a.iter().cloned().collect()))
        .collect();
    s.notes.clear();
    s.next_record = 0;
}

/// Notes the call and gives the script's answer (the Rust side's `Call` in the tests).
pub fn scripted(callee: usize, r: &mut Regs) {
    let (answer, record_at) = {
        let mut s = STATE.lock().unwrap();
        s.notes.push([
            callee as u32,
            r.eax,
            r.ebx,
            r.ecx,
            r.edx,
            r.esi,
            r.edi,
            r.ebp,
        ]);
        let answer = if s.notes.len() > LIMIT {
            // A flow the script does not end: everything says leave.
            let mut a = Answer::yes()
                .and(CAR_LEFT, 1)
                .and(LEAVE, 1)
                .and(ENDED_569, 1)
                .and(LEFT_A3C, 1);
            a.al = Some(1);
            a.zero = Some(false);
            if callee == FIND_RECORD {
                a.record = Answer::record(3, 0, 0, 0).record;
            }
            if callee == NEXT_RECORD_KIND {
                a.eax = Some(3);
            }
            a
        } else {
            s.script
                .iter_mut()
                .find(|(c, _)| *c == callee)
                .and_then(|(_, q)| q.pop_front())
                .unwrap_or_else(|| default_answer(callee))
        };
        let record_at = if answer.record.is_some() {
            let slot = s.next_record % RECORD_SLOTS;
            s.next_record += 1;
            s.records + (slot * RECORD_DWORDS * 4) as u32
        } else {
            0
        };
        (answer, record_at)
    };
    for &(at, v) in &answer.writes {
        set(at, v);
    }
    for &(n, offset, v) in &answer.player_writes {
        put(player(n) + offset, v);
    }
    if let Some(v) = answer.eax {
        r.eax = v;
    }
    if let Some(v) = answer.al {
        r.eax = (r.eax & !0xFF) | u32::from(v);
    }
    if let Some(v) = answer.edi {
        r.edi = v;
    }
    if let Some(c) = answer.carry {
        r.set_carry(c);
    }
    if let Some(z) = answer.zero {
        r.set_zero(z);
    }
    if let Some(rec) = answer.record {
        for (n, d) in rec.iter().enumerate() {
            put(record_at + 4 * n as u32, *d);
        }
        r.edi = record_at;
    }
}

/// When the script says nothing: the callees whose flags the flows test give them all
/// the same (no key, the speech over, the picture read).
fn default_answer(callee: usize) -> Answer {
    match callee {
        TAKE_KEY | WAIT_OR_KEY => Answer::key(0),
        STREAM_PLAYING => Answer::zero(false),
        LOAD_PICTURE_WITH_OVERLAY => Answer::carry(false),
        // A record that ends a championship (kind 3: its finish).
        FIND_RECORD => Answer::record(3, 0, 0, 0),
        _ => Answer::none(),
    }
}

extern "C" fn from_copy(regs: *mut Regs, place: u32) {
    let callee = STATE.lock().unwrap().callees[place as usize];
    scripted(callee, unsafe { &mut *regs });
}

macro_rules! flow_stand_ins {
    ($($name:ident = $place:expr),* $(,)?) => {
        $(
            #[unsafe(naked)]
            unsafe extern "C" fn $name() {
                core::arch::naked_asm!(
                    "pushfd",
                    "pushad",
                    "mov eax, esp",
                    "push {place}",
                    "push eax",
                    "call {act}",
                    "add esp, 8",
                    "popad",
                    "popfd",
                    "ret",
                    place = const $place,
                    act = sym from_copy,
                )
            }
        )*
        const STAND_INS: &[unsafe extern "C" fn()] = &[$($name),*];
    };
}

flow_stand_ins!(
    s0 = 0,
    s1 = 1,
    s2 = 2,
    s3 = 3,
    s4 = 4,
    s5 = 5,
    s6 = 6,
    s7 = 7,
    s8 = 8,
    s9 = 9,
    s10 = 10,
    s11 = 11,
    s12 = 12,
    s13 = 13,
    s14 = 14,
    s15 = 15,
    s16 = 16,
    s17 = 17,
    s18 = 18,
    s19 = 19,
    s20 = 20,
    s21 = 21,
    s22 = 22,
    s23 = 23,
    s24 = 24,
    s25 = 25,
    s26 = 26,
    s27 = 27,
    s28 = 28,
    s29 = 29,
    s30 = 30,
    s31 = 31,
    s32 = 32,
    s33 = 33,
    s34 = 34,
    s35 = 35,
    s36 = 36,
    s37 = 37,
    s38 = 38,
    s39 = 39,
    s40 = 40,
    s41 = 41,
    s42 = 42,
    s43 = 43,
    s44 = 44,
    s45 = 45,
    s46 = 46,
    s47 = 47,
    s48 = 48,
    s49 = 49,
    s50 = 50,
    s51 = 51,
    s52 = 52,
    s53 = 53,
    s54 = 54,
    s55 = 55,
    s56 = 56,
    s57 = 57,
    s58 = 58,
    s59 = 59,
    s60 = 60,
    s61 = 61,
    s62 = 62,
    s63 = 63,
);

/// An original to test: its address and length as IDA shows them, its first bytes as in
/// the image, the offsets of its `call rel32` (all go to stand-ins) and of its `0F 8x`
/// jumps to the fatal reporter (pointed at `install::stand_ins::fatal_landing`: only the
/// `pusha` is on the stack there), and the variables it writes (with those the scripts
/// write).
pub struct Flow<'a> {
    pub address: usize,
    pub len: usize,
    pub entry: &'a [u8],
    pub calls: &'a [usize],
    pub fatal_jumps: &'a [usize],
    pub writes: &'a [usize],
}

/// A case: its name, what it puts in place before each side (after the variables are put
/// back), its script, and the registers at entry.
pub struct Case<'a> {
    pub what: String,
    pub prepare: &'a dyn Fn(),
    pub script: Script,
    pub before: Regs,
}

/// What a Rust version is, for `test_flow`.
pub type Rust<'a> = &'a dyn Fn(&Regs, Call) -> Result<Regs, Refused>;

/// The variables the scripts write, compared and put back with the flow's.
const SCRIPT_WRITES: [usize; 6] = [ANSWER, CAR_LEFT, LEAVE, ENDED_569, LEFT_A3C, RACE_RESULT];

/// Runs each case on the copy and on the Rust version: the calls with eax .. ebp, the
/// registers left, the variables and the players compared, and whether it ended in the
/// fatal reporter (the Rust version's `FatalError`).
pub fn test_flow(
    mut t: SelfTest,
    f: &Flow,
    extra: &[Region],
    cases: &[Case],
    rust: Rust,
) -> (usize, usize) {
    use crate::install::stand_ins::{WENT_FATAL, fatal_landing};
    use std::sync::atomic::Ordering;
    let mut arena = vec![0u32; RECORD_SLOTS * RECORD_DWORDS];
    let callees = Mutex::new(Vec::<usize>::new());
    let original = game_addr(rva(f.address));
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(original, f.len, f.entry, f.calls, &|target| {
            let mut list = callees.lock().unwrap();
            let callee = target + 0x40_0000;
            let place = match list.iter().position(|&c| c == callee) {
                Some(p) => p,
                None => {
                    list.push(callee);
                    list.len() - 1
                }
            };
            STAND_INS.get(place).map(|&s| s as *const () as usize)
        })
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let callees = callees.into_inner().unwrap();
    if callees.len() > STAND_INS.len() {
        return t.skip("more callees than stand-ins");
    }
    for &offset in f.fatal_jumps {
        unsafe { copy.retarget(offset + 2, fatal_landing as *const () as usize) };
    }
    {
        let mut s = STATE.lock().unwrap();
        s.callees = callees;
        s.records = arena.as_mut_ptr() as u32;
    }
    let mut regions: Vec<Region> = f.writes.iter().map(|&v| at(rva(v))).collect();
    regions.extend(SCRIPT_WRITES.iter().map(|&v| at(rva(v))));
    regions.extend(crate::menu::player::player_regions());
    regions.extend_from_slice(extra);
    let saved = capture(&regions);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in cases {
        restore(&regions, &saved);
        (case.prepare)();
        begin(&case.script);
        WENT_FATAL.store(0, Ordering::SeqCst);
        let mut ro = case.before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let fatal = WENT_FATAL.load(Ordering::SeqCst) == 1;
        let theirs = (
            fatal,
            STATE.lock().unwrap().notes.clone(),
            if fatal { None } else { Some(regs(&ro)) },
            capture(&regions),
        );

        restore(&regions, &saved);
        (case.prepare)();
        begin(&case.script);
        let left = rust(&case.before, &mut |c, r| {
            scripted(c, r);
            Ok(())
        });
        let mine = (
            matches!(left, Err(Refused::Fatal(_))),
            STATE.lock().unwrap().notes.clone(),
            left.as_ref().ok().map(regs),
            capture(&regions),
        );
        t.check(
            theirs == mine && left.as_ref().err().is_none_or(|_| mine.0),
            || {
                let first = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
                format!(
                    "{}: fatal {} / {}; calls {} / {}, first differing {first:?}: {:x?} / {:x?}; \
                 registers {:x?} / {:x?}; rust {:?}; state: {}",
                    case.what,
                    theirs.0,
                    mine.0,
                    theirs.1.len(),
                    mine.1.len(),
                    first.and_then(|n| theirs.1.get(n)),
                    first.and_then(|n| mine.1.get(n)),
                    theirs.2,
                    mine.2,
                    left.as_ref().err(),
                    crate::service::harness::diff_summary(&theirs.3, &mine.3)
                )
            },
        );
    }
    restore(&regions, &saved);
    STATE.lock().unwrap().records = 0;
    drop(arena);
    t.finish()
}

/// A run of the player cars' choices for `n` players: the first `left` of them chosen,
/// the next one left (`dword_758806` 1), or all chosen when `left >= n`.
pub fn car_choices(n: usize, left: usize) -> Vec<Answer> {
    (0..n)
        .map(|k| {
            if k == left {
                Answer::write(CAR_LEFT, 1)
            } else {
                Answer::none()
            }
        })
        .collect()
}

/// An event flow's hook file: the hook (ungated, with the patch options given), its
/// dispatch (shadow mode runs the original alone; the Rust version with `game_call`) and
/// its self-test.
macro_rules! flow_hook {
    ($name:literal, $address:literal, $rust:path, $test:path $(, copy($len:expr, $calls:expr))?) => {
        static ORIGINAL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

        pub static HOOK: $crate::framework::RawHook = {
            let hook = $crate::framework::RawHook::new(
                $name,
                $crate::framework::rva($address),
                thunk,
                &ORIGINAL,
            );
            $(let hook = $crate::menu::event_flow::copied(hook, $len, $calls);)?
            hook.self_test(self_test)
        };

        $crate::framework::hook_asm!(
            thunk,
            call_original,
            dispatch = dispatch,
            original = ORIGINAL,
            ungated
        );

        extern "C" fn dispatch(r: &mut $crate::framework::Regs) {
            HOOK.count_call();
            if $crate::framework::shadow() {
                unsafe { call_original(r) };
                return;
            }
            match unsafe { $rust(r, &mut $crate::menu::event_flow::game_call) } {
                Ok(left) => *r = $crate::framework::Regs { esp: r.esp, ..left },
                Err(e) => unsafe { $crate::fatal::raise::refused(&HOOK, e, r, call_original) },
            }
        }

        fn self_test() -> (usize, usize) {
            let Some(t) = $crate::framework::SelfTest::begin(&HOOK) else {
                return (0, 0);
            };
            $test(t)
        }
    };
}
pub(crate) use flow_hook;

/// For `flow_hook`: a private copy of `len` bytes with its calls relocated and the jump
/// written by hand (a call in the first bytes).
pub const fn copied(
    hook: crate::framework::RawHook,
    len: usize,
    calls: &'static [usize],
) -> crate::framework::RawHook {
    hook.private_copy(len).relocate_calls(calls).manual_patch()
}
