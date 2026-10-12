//! `menu::set_up_front_end` (0x47C63F): what the main menu's loop does first.
//!
//! In:  nothing (ebp and, after the loading picture, ecx are passed on as they are)
//! Out: every register preserved
//! In this order:
//! - `menu::add_clio_car` if `g_hasClio` is 1; `menu::link_player_names`; the loading
//!   picture (`sub_47CEA8`, not ported); `gfx::build_menu_sprites`;
//!   `menu::fit_player_names` (with the ecx the loading picture left);
//! - `dword_5180FC` = `dword_755688` and `dword_518100` = `dword_75568C`;
//! - `saves::allocate_best_times`, and the nine tables' addresses copied to the variables
//!   the screens read them from (`COPIES`);
//! - unless the best times were loaded (`g_bestTimesLoaded`): the six fillers of made-up
//!   times, `dword_755A0C` (the checkpoints' switch) set before each (1 only for
//!   `default_checkpoint_times`), then `g_bestTimesLoaded` = 1;
//! - unless the options were loaded (`g_optionsLoaded`): `records::default_stage_records`
//!   on each of the thirteen record tables.
//!
//! One caller, `menu::main_menu_loop`.
//! Shadow mode runs the original alone: its parts are compared in their own hooks.
//! Self-test: a redirected copy whose thirteen kinds of call go to the test.
//! Refuses what its parts refuse; such a refusal comes with the work half done.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::saves::default_times::{WITH_CHECKPOINTS_RVA, to_the_log};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::set_up_front_end", rva(0x47C63F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const HAS_CLIO_RVA: usize = rva(0x529DB4); // g_hasClio
const BEST_TIMES_LOADED_RVA: usize = rva(0x747614); // g_bestTimesLoaded
const OPTIONS_LOADED_RVA: usize = rva(0x75557D); // g_optionsLoaded
/// (from, to): two dwords, then the nine tables of `saves::allocate_best_times`.
const COPIES: [(usize, usize); 11] = [
    (0x755688, 0x5180FC),
    (0x75568C, 0x518100),
    (0x747618, 0x7556F4),
    (0x747620, 0x7556D4),
    (0x747628, 0x7556D8),
    (0x747630, 0x7556DC),
    (0x747638, 0x7556E0),
    (0x747640, 0x7556EC),
    (0x747648, 0x7556F0),
    (0x747650, 0x7556E4),
    (0x747658, 0x7556E8),
];
/// The record tables that get made-up records: the championship's and twelve more.
const RECORD_TABLES: [usize; 13] = [
    0x735576, 0x739642, 0x7391C2, 0x738D42, 0x7388C2, 0x738442, 0x737FC2, 0x739AC2, 0x739F42,
    0x73A3C2, 0x73A842, 0x73ACC2, 0x73B142,
];

// ---- Rust version ----

/// What the function calls, in the order of the test's stand-ins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    AddClioCar,
    LinkPlayerNames,
    LoadingPicture,
    BuildMenuSprites,
    FitPlayerNames,
    AllocateBestTimes,
    DefaultBestScores,
    DefaultChampionshipTimes,
    DefaultCheckpointTimes,
    DefaultClassTimes,
    DefaultStageTimes,
    DefaultStageTimesAlt,
    DefaultStageRecords,
}

/// Does a call with the registers as they are at it (edi is the table for
/// `DefaultStageRecords`); the loading picture may change them.
pub type Outside<'a> = &'a mut dyn FnMut(Call, &mut Regs) -> Result<(), Refused>;

pub unsafe fn set_up_front_end(r: &mut Regs, outside: Outside) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(HAS_CLIO_RVA) == 1 {
            outside(Call::AddClioCar, r)?;
        }
        outside(Call::LinkPlayerNames, r)?;
        outside(Call::LoadingPicture, r)?;
        outside(Call::BuildMenuSprites, r)?;
        outside(Call::FitPlayerNames, r)?;
        for (n, (from, to)) in COPIES.into_iter().enumerate() {
            if n == 2 {
                outside(Call::AllocateBestTimes, r)?;
            }
            game_write(rva(to), game_read::<u32>(rva(from)));
        }
        if game_read::<u32>(BEST_TIMES_LOADED_RVA) != 1 {
            for (call, with_checkpoints) in [
                (Call::DefaultBestScores, 0u32),
                (Call::DefaultChampionshipTimes, 0),
                (Call::DefaultCheckpointTimes, 1),
                (Call::DefaultClassTimes, 0),
                (Call::DefaultStageTimes, 0),
                (Call::DefaultStageTimesAlt, 0),
            ] {
                game_write(WITH_CHECKPOINTS_RVA, with_checkpoints);
                outside(call, r)?;
            }
            game_write(BEST_TIMES_LOADED_RVA, 1u32);
        }
        if game_read::<u32>(OPTIONS_LOADED_RVA) != 1 {
            for table in RECORD_TABLES {
                r.edi = game_addr(rva(table)) as u32;
                outside(Call::DefaultStageRecords, r)?;
            }
        }
        Ok(())
    }
}

// ---- hook ----

/// The calls done by the Rust versions; the loading picture by the game, and the
/// sprites through their hook.
fn in_game(call: Call, r: &mut Regs) -> Result<(), Refused> {
    use crate::saves::{
        allocate_best_times, default_best_scores, default_championship_times,
        default_checkpoint_times, default_class_times, default_stage_times,
        default_stage_times_alt,
    };
    unsafe {
        match call {
            Call::AddClioCar => crate::menu::add_clio_car::add_clio_car()?,
            Call::LinkPlayerNames => crate::menu::link_player_names::link_player_names()?,
            Call::LoadingPicture => call_game(r, game_addr(rva(0x47CEA8))),
            // Through its own hook, which decides which version of it runs.
            Call::BuildMenuSprites => call_game(&mut { *r }, game_addr(rva(0x47C4A1))),
            Call::FitPlayerNames => crate::menu::fit_player_names::fit_player_names(r.ecx)?,
            Call::AllocateBestTimes => allocate_best_times::allocate_best_times(
                game_addr(allocate_best_times::VARS_RVA) as *mut u32,
                game_addr(rva(0x7475D0)) as *mut crate::memory::MemoryList,
                game_addr(rva(0x735576)) as *const u8,
            )?,
            Call::DefaultBestScores => default_best_scores::default_best_scores()?,
            Call::DefaultChampionshipTimes => {
                default_championship_times::default_championship_times(r.ebp, &mut to_the_log)?
            }
            Call::DefaultCheckpointTimes => {
                default_checkpoint_times::default_checkpoint_times(r.ebp, &mut to_the_log)?
            }
            Call::DefaultClassTimes => default_class_times::default_class_times()?,
            Call::DefaultStageTimes => {
                default_stage_times::default_stage_times(r.ebp, &mut to_the_log)?
            }
            Call::DefaultStageTimesAlt => {
                default_stage_times_alt::default_stage_times_alt(r.ebp, &mut to_the_log)?
            }
            Call::DefaultStageRecords => {
                crate::records::default_stage_records::default_stage_records(r.edi)?
            }
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let mut running = *r;
    if let Err(e) = unsafe { set_up_front_end(&mut running, &mut in_game) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

const LEN: usize = 0x1A1;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x83, 0x3D, 0xB4, 0x9D, 0x52, 0x00, 0x01];
const CALLS: [usize; 25] = [
    0xA, 0xF, 0x14, 0x19, 0x1E, 0x37, 0xA9, 0xB8, 0xC7, 0xD6, 0xE5, 0xF4, 0x116, 0x121, 0x12C,
    0x137, 0x142, 0x14D, 0x158, 0x163, 0x16E, 0x179, 0x184, 0x18F, 0x19A,
];
/// The callees, in the order of `Call`.
const CALLEES: [usize; 13] = [
    0x471C33, 0x47059E, 0x47CEA8, 0x47C4A1, 0x411445, 0x47C4D8, 0x47C923, 0x47C973, 0x47CA46,
    0x47C8E3, 0x47C7FE, 0x47C86F, 0x47C7E0,
];
const ALL: [Call; 13] = [
    Call::AddClioCar,
    Call::LinkPlayerNames,
    Call::LoadingPicture,
    Call::BuildMenuSprites,
    Call::FitPlayerNames,
    Call::AllocateBestTimes,
    Call::DefaultBestScores,
    Call::DefaultChampionshipTimes,
    Call::DefaultCheckpointTimes,
    Call::DefaultClassTimes,
    Call::DefaultStageTimes,
    Call::DefaultStageTimesAlt,
    Call::DefaultStageRecords,
];

/// Each call: its place, ecx, ebp, edi, and the checkpoints' switch as it was.
static CALLED: Mutex<Vec<[u32; 5]>> = Mutex::new(Vec::new());

/// Notes the call; the loading picture leaves other registers, as the real one may.
fn handler(place: u32, r: &mut Regs) {
    let switch: u32 = unsafe { game_read(WITH_CHECKPOINTS_RVA) };
    CALLED
        .lock()
        .unwrap()
        .push([place, r.ecx, r.ebp, r.edi, switch]);
    if place == Call::LoadingPicture as u32 {
        (r.eax, r.ecx, r.ebp) = (0x1111, 0x2222, 0x3333);
    }
}

fn through_the_test(call: Call, r: &mut Regs) -> Result<(), Refused> {
    let place = ALL.iter().position(|&known| known == call).unwrap() as u32;
    act(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x47C63F)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                CALLEES
                    .iter()
                    .position(|&known| rva(known) == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let mut regions: Vec<Region> = COPIES.iter().map(|&(_, to)| at(rva(to))).collect();
    regions.extend([at(WITH_CHECKPOINTS_RVA), at(BEST_TIMES_LOADED_RVA)]);
    let mut inputs: Vec<Region> = COPIES.iter().map(|&(from, _)| at(rva(from))).collect();
    inputs.extend([at(HAS_CLIO_RVA), at(OPTIONS_LOADED_RVA)]);
    let saved = (capture(&regions), capture(&inputs));
    // (g_hasClio, g_bestTimesLoaded, g_optionsLoaded)
    for (clio, times, options) in [
        (0u32, 1u32, 1u32),
        (1, 1, 1),
        (0, 0, 1),
        (0, 1, 0),
        (1, 0, 0),
        (2, 2, 2),
    ] {
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            for (n, (from, _)) in COPIES.into_iter().enumerate() {
                game_write(rva(from), 0x5000_0000u32 + n as u32);
            }
            game_write(HAS_CLIO_RVA, clio);
            game_write(BEST_TIMES_LOADED_RVA, times);
            game_write(OPTIONS_LOADED_RVA, options);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let ended = if rust {
                let mut running = before;
                set_up_front_end(&mut running, &mut through_the_test).is_err() as u32
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                !ro.preserved_except(&before, 0) as u32
            };
            (ended, capture(&regions), CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 == 0, || {
            format!(
                "clio {clio}, times loaded {times}, options loaded {options}: original ended {} with {} calls; rust ended {} with {} calls; first different call {:?}; first different byte left {:?}",
                theirs.0,
                theirs.2.len(),
                mine.0,
                mine.2.len(),
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b),
                theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b),
            )
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
