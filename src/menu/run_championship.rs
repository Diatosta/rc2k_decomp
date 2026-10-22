//! `menu::run_championship` (0x4A0E60): a whole championship (event type 4), from its
//! start or from a saved game (`dword_755970` 1, which also makes `dword_755950` 2), to
//! its end. In order:
//!   - `dword_755250` 0; unless continuing: `menu::begin_event`, `shuffle_drivers`,
//!     `clear_result_slots`, `reset_drivers`, `reset_driver_condition`, `dword_755254` 0,
//!     `clear_players_284`; then `take_player1_units`, `reset_cars_750724`;
//!     `g_yesNoAnswer`, `dword_755561`, `dword_75555D` 0, `g_eventMode` 2, `dword_75AB40`
//!     2; with no section to start at (`dword_7554C0` 0) the welcome
//!     (`menu::championship_welcome`, or `menu::a8_welcome` were `g_eventMode` not 2);
//!     `sound::stop_stream`; a yes ends it; `dword_755970` 0;
//!   - each section: `dword_755A3C`, `dword_75593C`, `dword_755938`, `dword_755928` 0,
//!     `dword_75592C` 1, `dword_755250` 0, `section` `dword_7554C0` - 1 when that is set;
//!   - each record of the section (`records::find_record` on `g_recordTable`, `section`,
//!     `index`; `dword_75AB20` 0 first) by its kind (+0): 3, the championship's end
//!     (`dword_755928` 1, then the standings below); 5, a welcome (`dword_755924` its
//!     +0x38, `index` + 1, `dword_755934` 1, `begin_event`, `menu::welcome_loop`); 4, the
//!     section's end: the championship standings (`standings_b_from_all`, sorted, the stage
//!     points, sorted twice, `class_championship_pass`), and unless the section was given
//!     (`dword_7554C0`) or it was the end, the overall points, the table numbered and shown
//!     (`show_championship_standings`), `section_finish`, `review_options_480d46_loop`,
//!     and on to the next section (`begin_event`); 1, a service due (`dword_755938` 1,
//!     `dword_75593C` its +0x14, `dword_755940` its +0xC, `dword_755944` and
//!     `dword_75594C` 1, `dword_755254` + 1, `index` + 1); 0, a stage: the rally's stages
//!     (`rally_stages_loop`), the stage (`next_stage_loop`), the service (`service_pass`),
//!     the races (`run_event_races`, unless `dword_755250`), the times, the stage's and
//!     (from the second stage on, `dword_755950` >= 1) the rally's results, the record
//!     bests (`keep_record_best`, `record_entries_pass`, `best_times_loop` when
//!     `dword_755920`), then by `menu::next_record_kind`: 3 the next stage at once, 4 the
//!     section's end, 1 a service next (`dword_755944`, `dword_75594C` 1), else the
//!     inspection (`inspection_pass`, `dword_75594C` 0); then the review
//!     (`review_options_47d5a5_loop` when `dword_75AAD8` or `dword_75594C`, else
//!     `review_options_48108e_loop`) and the next record (`index`, `dword_755950` + 1,
//!     `dword_755254` 0). A yes on the stage's screens resets the players' times
//!     (`dword_755250` 1) and goes to the rally's results (`dword_75AB20` 1, the leave
//!     flags and `dword_75592C` 0, `walk_section_records`, the rally's tables shown) and
//!     then the section's end; any other kind is fatal error 0x8070DBA0 "Invalid Level
//!     Type";
//!   - the end: unless the section was given, `championship_finish`, `clear_score_marks`,
//!     `best_class_scores`, `best_overall_scores`; `review_options_481429_loop`;
//!   - a yes to leaving on most screens ends it at once; at the very end, with the
//!     registers of the entry, `menu::show_in_game_options`.
//!
//! In:  nothing
//! Out: every register as `menu::show_in_game_options` leaves the entry's (all kept)
//!
//! `run_event` is shared with `menu::run_a8_championship` (`a8`: its differences). Callers:
//! `saves::load_savegame` and the five starters (`menu::start_championship_*`). The calls
//! through `event_flow::Call` (in the game `game_call`). Ungated (a whole event). Refuses
//! what its callees refuse, when they are reached (the original then runs from the start).
//! Shadow mode runs the original alone. Self-test: `event_flow::test_flow`, record kinds
//! and the screens' answers by script.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::menu::event_flow::*;
use crate::menu::flow_game::Game;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::run_championship", rva(0x4A0E60), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

pub const STAGES_DONE: usize = 0x755950; // dword_755950
pub const RESET_TIMES: usize = 0x755250; // dword_755250
pub const SERVICES: usize = 0x755254; // dword_755254
pub const MODE_AB40: usize = 0x75AB40; // dword_75AB40
pub const START_SECTION: usize = 0x7554C0; // dword_7554C0 (the A8's dword_7554C4)
pub const START_SECTION_A8: usize = 0x7554C4;
pub const SERVICE_TIME: usize = 0x75593C; // dword_75593C
pub const SERVICE_DUE: usize = 0x755938; // dword_755938
pub const THE_END: usize = 0x755928; // dword_755928
pub const GOES_ON: usize = 0x75592C; // dword_75592C
pub const RALLY_DONE: usize = 0x75AB20; // dword_75AB20
pub const WELCOME_FIGURE: usize = 0x755924; // dword_755924
pub const WELCOMED: usize = 0x755934; // dword_755934
pub const SERVICE_940: usize = 0x755940; // dword_755940
pub const SERVICE_944: usize = 0x755944; // dword_755944
pub const SERVICE_NEXT: usize = 0x75594C; // dword_75594C
pub const BEST_TIMES: usize = 0x755920; // dword_755920
pub const REVIEW_KIND: usize = 0x75AAD8; // dword_75AAD8

/// Every variable the two drivers write or the tests set.
pub const VARIABLES: [usize; 30] = [
    CONTINUING,
    STAGES_DONE,
    RESET_TIMES,
    SERVICES,
    ANSWER,
    LEAVE_561,
    LEAVE,
    MODE,
    MODE_AB40,
    LEFT_A3C,
    SERVICE_TIME,
    SERVICE_DUE,
    THE_END,
    GOES_ON,
    SECTION,
    INDEX,
    RALLY_DONE,
    WELCOME_FIGURE,
    WELCOMED,
    SERVICE_940,
    SERVICE_944,
    SERVICE_NEXT,
    CAR_LEFT,
    START_SECTION,
    START_SECTION_A8,
    BEST_TIMES,
    REVIEW_KIND,
    ENDED_569,
    PLAYER_COUNT,
    CURRENT,
];

/// The two championships' driver: `a8` for `menu::run_a8_championship`'s differences.
pub unsafe fn run_event(entry: &Regs, call: Call, a8: bool) -> Result<Regs, Refused> {
    let mut game = Game::new(entry, call);
    // Converted: `rc2k_core::menu::championship::run`.
    rc2k_core::menu::championship::run(&mut game, a8)?;
    Ok(game.left())
}

pub unsafe fn run_championship(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    unsafe { run_event(entry, call, false) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { run_championship(r, &mut game_call) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { run_championship(r, &mut game_call) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

const CALLS: &[usize] = &[
    0x27, 0x2C, 0x31, 0x36, 0x3B, 0x4A, 0x4F, 0x54, 0x9D, 0xA4, 0xA9, 0x130, 0x163, 0x168, 0x185,
    0x18A, 0x18F, 0x194, 0x199, 0x19E, 0x1C8, 0x1CD, 0x1D2, 0x1D7, 0x1DC, 0x1E1, 0x1E6, 0x205,
    0x217, 0x267, 0x2C3, 0x2DB, 0x2EC, 0x304, 0x315, 0x32D, 0x37D, 0x382, 0x387, 0x39A, 0x39F,
    0x3A4, 0x3C1, 0x3E7, 0x3F9, 0x3FE, 0x403, 0x41B, 0x42F, 0x434, 0x439, 0x451, 0x472, 0x497,
    0x49C, 0x4B4, 0x4D2, 0x4D7, 0x4DC, 0x4F4, 0x512, 0x517, 0x525, 0x53D, 0x55B, 0x597, 0x5AF,
    0x5EC, 0x5F3, 0x639, 0x647, 0x64C, 0x65A, 0x65F, 0x665,
];

/// The scripts both championships' tests run: (continuing, the section given, stages
/// done, `dword_755920`, `dword_75AAD8`, the script).
pub fn scripts() -> Vec<(u32, u32, u32, u32, u32, Script)> {
    let rec = Answer::record;
    let none = Answer::none;
    let yes = Answer::yes;
    let kind = Answer::eax;
    vec![
        // the welcome's yes
        (
            0,
            0,
            0,
            0,
            0,
            vec![
                (CHAMPIONSHIP_WELCOME, vec![yes()]),
                (A8_WELCOME, vec![yes()]),
            ],
        ),
        // the stop's yes (a saved game going on: the welcome returns at once)
        (1, 0, 0, 0, 0, vec![(STOP_STREAM, vec![yes()])]),
        // a service, a welcome page left
        (
            1,
            2,
            0,
            0,
            0,
            vec![
                (
                    FIND_RECORD,
                    vec![rec(1, 0x111, 0x222, 0), rec(5, 0, 0, 0x333)],
                ),
                (WELCOME_LOOP, vec![yes()]),
            ],
        ),
        // a stage, its stages page answered yes: the rally's results, left there
        (
            0,
            0,
            0,
            0,
            0,
            vec![
                (FIND_RECORD, vec![rec(0, 0, 0, 0)]),
                (RALLY_STAGES_LOOP, vec![yes()]),
                (SHOW_OVERALL_RALLY_RESULTS, vec![yes()]),
                (CLASS_RALLY_RESULTS_PASS, vec![none(), yes()]),
            ],
        ),
        // a stage run through, an inspection, the review left
        (
            0,
            0,
            1,
            1,
            0,
            vec![
                (FIND_RECORD, vec![rec(0, 0, 0, 0)]),
                (NEXT_RECORD_KIND, vec![kind(3), kind(2)]),
                (REVIEW_48108E, vec![yes()]),
            ],
        ),
        // a stage, a service next, the review, the section's end, its review left
        (
            0,
            0,
            0,
            0,
            0,
            vec![
                (FIND_RECORD, vec![rec(0, 0, 0, 0), rec(4, 0, 0, 0)]),
                (NEXT_RECORD_KIND, vec![kind(1)]),
                (REVIEW_480D46, vec![Answer::write(ENDED_569, 1)]),
            ],
        ),
        // the championship's end (the default record), each finish screen
        (0, 0, 0, 0, 0, vec![]),
        (0, 0, 0, 0, 0, vec![(CHAMPIONSHIP_FINISH, vec![yes()])]),
        (0, 0, 0, 0, 0, vec![(BEST_CLASS_SCORES, vec![yes()])]),
        // a section given: its end goes to the finish without its screens
        (0, 3, 0, 0, 0, vec![(FIND_RECORD, vec![rec(4, 0, 0, 0)])]),
        // an unknown kind: the fatal error
        (0, 0, 0, 0, 0, vec![(FIND_RECORD, vec![rec(7, 0, 0, 0)])]),
        // the races say it does not go on
        (
            0,
            0,
            0,
            0,
            0,
            vec![
                (FIND_RECORD, vec![rec(0, 0, 0, 0)]),
                (RUN_EVENT_RACES, vec![Answer::write(GOES_ON, 0)]),
            ],
        ),
        // the next record ends the section, the standings left
        (
            0,
            0,
            2,
            0,
            1,
            vec![
                (FIND_RECORD, vec![rec(0, 0, 0, 0)]),
                (NEXT_RECORD_KIND, vec![kind(4), kind(4)]),
                (CLASS_CHAMPIONSHIP_PASS, vec![yes()]),
            ],
        ),
        // the best times left, then the inspection left
        (
            0,
            0,
            0,
            1,
            0,
            vec![
                (FIND_RECORD, vec![rec(0, 0, 0, 0), rec(0, 0, 0, 0)]),
                (BEST_TIMES_LOOP, vec![yes(), none()]),
                (INSPECTION_PASS, vec![yes()]),
                (NEXT_RECORD_KIND, vec![kind(0)]),
                (SECTION_FINISH, vec![yes()]),
            ],
        ),
        // two stages, the second's review with `dword_75AAD8`, then the end
        (
            0,
            0,
            0,
            0,
            1,
            vec![
                (FIND_RECORD, vec![rec(0, 0, 0, 0), rec(0, 0, 0, 0)]),
                (NEXT_RECORD_KIND, vec![kind(0), kind(5), kind(3)]),
                (SHOW_OVERALL_STAGE_RESULTS, vec![none(), yes()]),
            ],
        ),
    ]
}

/// One championship's test: `scripts` as cases.
pub fn test_event(t: SelfTest, flow: &Flow, a8: bool) -> (usize, usize) {
    let all = scripts();
    let prepares: Vec<Box<dyn Fn()>> = all
        .iter()
        .map(|&(continuing, section, done, best, review, _)| {
            Box::new(move || {
                set(CONTINUING, continuing);
                set(if a8 { START_SECTION_A8 } else { START_SECTION }, section);
                set(STAGES_DONE, done);
                set(BEST_TIMES, best);
                set(REVIEW_KIND, review);
                for v in [ENDED_569, LEFT_A3C, SECTION, INDEX, ANSWER] {
                    set(v, 0);
                }
            }) as Box<dyn Fn()>
        })
        .collect();
    let cases: Vec<Case> = all
        .into_iter()
        .zip(&prepares)
        .enumerate()
        .map(|(n, ((cont, section, _, _, _, script), prepare))| Case {
            what: format!("script {n} (continuing {cont}, section {section})"),
            prepare: prepare.as_ref(),
            script,
            before: Regs::SENTINEL,
        })
        .collect();
    test_flow(t, flow, &[], &cases, &|r, c| unsafe { run_event(r, c, a8) })
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let flow = Flow {
        address: 0x4A0E60,
        len: 0x66B,
        // pusha; cmp dword_755970, 1
        entry: &[0x60, 0x83, 0x3D, 0x70, 0x59, 0x75, 0x00, 0x01],
        calls: CALLS,
        fatal_jumps: &[0x2BD],
        writes: &VARIABLES,
    };
    test_event(t, &flow, false)
}
