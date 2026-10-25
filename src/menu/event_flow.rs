//! What the event flows share (IDA, 2026-10-19): the championship drivers
//! (`menu::run_championship`, `menu::run_a8_championship`), their starters, the single
//! rally's and the time trial's multi-player events. Each of them is a sequence of calls
//! and of tests of the game's variables. The flows run on `menu::flow_game::Game`
//! (the callees' Rust versions, the screens through `menu::screen_loop`), and carry the
//! registers the originals hand from call to call in a `Flow`.

use crate::errors::BadInput;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::menu::screen_loop::Carried;

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
pub const ROUND: usize = 0x758714; // dword_758714: the rounds of a multi-player event

/// The car lists: `off_759CAE` (the championship's), `off_759D16` (the A8 championship's),
/// `off_759DCA` / `off_75A042` (by `dword_7554D8`), `off_75A412` (with cheat bit 1).
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
