//! What the event flows share (IDA, 2026-10-19): the championship drivers
//! (`menu::run_championship`, `menu::run_a8_championship`), their starters, the single
//! rally's and the time trial's multi-player events. Each of them is a sequence of calls
//! and of tests of the game's variables. The flows run on `menu::flow_game::Game`
//! (the callees' Rust versions, the screens through `menu::screen_loop`), and carry the
//! registers the originals hand from call to call in a `Flow`.

use crate::errors::BadInput;
use crate::menu::captions_store::CAPTIONS;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::cheats_store::CHEAT_VARS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::players_store::PLAYER_TABLE;
use crate::menu::results_store::RESULT_FIGURES;
use crate::menu::screen_loop::Carried;
use crate::session::Dword;

/// A game variable.
pub fn get(v: &Dword) -> u32 {
    v.get()
}

pub fn set(v: &Dword, x: u32) {
    v.set(x);
}

pub fn inc(v: &Dword) {
    set(v, get(v).wrapping_add(1));
}

pub fn dword(p: u32) -> u32 {
    unsafe { (p as *const u32).read_unaligned() }
}

pub fn put(p: u32, v: u32) {
    unsafe { (p as *mut u32).write_unaligned(v) };
}

// ---- the variables ----

pub const ANSWER: &Dword = &MENU_PAGES.yes_no_answer; // g_yesNoAnswer
pub const CURRENT: &Dword = &EVENT_STATE.current_player; // dword_75598C: the current player
pub const PLAYER_COUNT: &Dword = &EVENT_STATE.player_count; // dword_755984
pub const EVENT_MODE: &Dword = &EVENT_STATE.event_kind; // dword_755988 (9 .. 0x18, `menu::run_event_races`)
pub const EVENT_TYPE: &Dword = &EVENT_STATE.event_type; // dword_755954: 3 time trial, 4, 5 the championships
pub const MODE: &Dword = &CHAMP_VIEW.event_mode; // g_eventMode: 2, 3, 0x13
pub const CAR_LEFT: &Dword = &CAPTIONS.screen_left; // dword_758806: the car choice was left
pub const LEAVE: &Dword = &LOOP_FLAGS.went_back; // dword_75555D
pub const LEAVE_561: &Dword = &LOOP_FLAGS.back; // dword_755561
pub const ENDED_565: &Dword = &LOOP_FLAGS.ended_b; // dword_755565
pub const ENDED_569: &Dword = &LOOP_FLAGS.ended_a; // dword_755569
pub const DONE: &Dword = &EVENT_STATE.event_done; // dword_755980: 1 when an event is over
pub const LEFT_A3C: &Dword = &DRAW_WORK.escaped; // dword_755A3C
pub const CONTINUING: &Dword = &EVENT_STATE.continuing_save; // dword_755970: 1 when a saved game goes on
pub const INDEX: &Dword = &EVENT_STATE.index; // index
pub const SECTION: &Dword = &EVENT_STATE.section; // section
pub const CHEATS: &Dword = &CHEAT_VARS.cheats; // g_cheats
pub const ROUND: &Dword = &RESULT_FIGURES.round; // dword_758714: the rounds of a multi-player event

/// The car lists: `off_759CAE` (the championship's), `off_759D16` (the A8 championship's),
/// `off_759DCA` / `off_75A042` (by `dword_7554D8`), `off_75A412` (with cheat bit 1).
pub const CAR_LIST_KIND: &Dword = &SCREEN_FLOW.car_list_kind; // dword_7554D8: 1 picks the first list

/// Player `n`'s record (`g_players`).
pub fn player(n: u32) -> u32 {
    PLAYER_TABLE.pointers[n as usize].get()
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
pub const STAGE_KIND: &Dword = &EVENT_STATE.stage_group; // dword_7558C8
pub const STAGE_SUB: &Dword = &EVENT_STATE.stage_sub; // dword_7558D0
pub const STAGE_NUMBER: &Dword = &EVENT_STATE.stage_number; // dword_7558CC
const KINDS: u32 = 4;
pub const STAGE_RECORD: u32 = 0xA4;
/// The stages' best times: `dword_7556E4` (kind 1) or `dword_7556E8`, 0x618 a stage (its
/// +0x3C), ten entries of 0x9C each.
pub const TIMES_ONE: &Dword = &CHAMP_VIEW.stage_best_times_alt; // dword_7556E4
pub const TIMES_OTHER: &Dword = &CHAMP_VIEW.time_trial_best_times; // dword_7556E8
pub const TIMES_LEN: u32 = 0x618;
pub const TIME_ENTRY: u32 = 0x9C;

/// The chosen stage's record; refused for a kind past the table's four or a null pointer
/// on the way (the original reads through it).
pub fn stage_record() -> Result<u32, BadInput> {
    let kind = get(STAGE_KIND);
    if kind >= KINDS {
        return Err(BadInput("a stage kind past off_735512's four"));
    }
    let rallies = crate::records::stages_store::STAGE_TABLES.group_list(kind);
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

// ---- the registers the flows carry ----

/// The registers an event flow carries from one callee to the next (the originals are
/// `pusha` .. `popa` bodies whose registers go on to every call): the values the screens
/// are run with (`Carried`: eax `arg`, ebx `palette`, ecx `measure_x`, edx `clock`, esi
/// `cars`, the carry), the zero flag, and edi, the record the last search or walk left
/// (`record`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Flow {
    pub c: Carried,
    pub zero: bool,
    pub record: u32,
}

impl Flow {
    /// The flow of a caller whose carried values are `c` (flags and edi clear, as the
    /// menus' items get them).
    pub fn new(c: &Carried) -> Self {
        Self {
            c: *c,
            zero: false,
            record: 0,
        }
    }
}
