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

pub fn dword(p: u32) -> Result<u32, crate::errors::BadInput> {
    crate::pointers::GamePtr::new(p).read::<u32>("a game value")
}

/// Reads for closures that a core function calls (they cannot return an error): the first
/// refusal is kept and the reads after it answer 0; the caller runs `check` right after the
/// core call, before it uses anything the core made.
#[derive(Default)]
pub struct Fault(std::cell::Cell<Option<BadInput>>);

impl Fault {
    /// The value, or the type's default after keeping the refusal.
    pub fn ok<T: Default>(&self, r: Result<T, BadInput>) -> T {
        r.unwrap_or_else(|e| {
            if self.0.get().is_none() {
                self.0.set(Some(e));
            }
            T::default()
        })
    }

    pub fn get(&self, p: u32) -> u32 {
        self.ok(dword(p))
    }

    /// A `T` of the game's memory at `p`.
    pub fn read<T: Copy + Default>(&self, p: u32, what: &str) -> T {
        self.ok(crate::pointers::GamePtr::new(p).read::<T>(what))
    }

    pub fn put(&self, p: u32, v: u32) {
        self.ok(put(p, v));
    }

    pub fn check(&self) -> Result<(), BadInput> {
        self.0.take().map_or(Ok(()), Err)
    }
}

/// `len` bytes of the game's memory from `first` for a core function that walks a table in
/// place; refused unless they all lie in one block (`pointers::region_of`). Keep it for
/// one step: a block can be given back later.
pub fn game_slice<'a>(first: u32, len: u32, what: &str) -> Result<&'a mut [u8], BadInput> {
    match crate::pointers::region_of(first) {
        Some((_, end)) if (end - first) >= len => {
            // SAFETY: all `len` bytes lie in one live region (a registered block or a moved
            // store); the caller is the only one using them for the step.
            Ok(unsafe { std::slice::from_raw_parts_mut(first as usize as *mut u8, len as usize) })
        }
        _ => {
            crate::framework::log(&format!(
                "pointers: {what} at {first:#010x} ({len} bytes) is outside every allocation"
            ));
            Err(BadInput(
                "a game pointer outside every allocation (rc2k.log names it)",
            ))
        }
    }
}

/// A refusal where the caller's signature cannot carry one (a trait of the core's, an
/// installer helper): the program ends naming it, as a refusal does at the top.
pub fn refuse_here<T>(r: Result<T, BadInput>, at: &str) -> T {
    match r {
        Ok(value) => value,
        Err(e) => crate::fatal::raise::end(at, e),
    }
}

pub fn put(p: u32, v: u32) -> Result<(), crate::errors::BadInput> {
    crate::pointers::GamePtr::new(p).write(v, "a game value")
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
        dword(rallies.wrapping_add(get(STAGE_SUB).wrapping_mul(4)))?
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
    Ok(base.wrapping_add(dword(record + 0x3C)?.wrapping_mul(TIMES_LEN)))
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
