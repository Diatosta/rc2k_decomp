//! `menu::walk_section_records` (0x471F0D): the current section's records walked from
//! `index`: a record tagged 0 gets the computer drivers' times (`menu::calc_cpu_times`)
//! and `dword_755944` 0, one tagged 1 the drivers' condition reset
//! (`menu::reset_driver_condition`) and `dword_755944` 1; past each of those and of any
//! other tag but 3 and 4, `index` and `dword_755950` up by one and the players' totals
//! reset (`menu::reset_player_totals`). Tag 4 ends the walk, tag 3 too with
//! `dword_755928` 1; then the drivers' condition reset.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_4A0E60`, `sub_4AA4D7`. The steps are their Rust versions. Refuses what
//! they refuse for any record of the walk, and a walk past 64 records, before anything.
//! Shadow mode runs the original alone (it logs, in `calc_cpu_times`). Self-test:
//! the original and the Rust version on the game's tables from the section's
//! first records; put back.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use rc2k_core::menu::section_walk::Step;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::walk_section_records",
    rva(0x471F0D),
    thunk,
    &ORIGINAL,
)
.private_copy(0x71)
.relocate_calls(&CALLS)
.manual_patch()
.self_test(self_test);

/// Its calls (a private copy: the loop goes back to +1, inside the patched bytes).
const CALLS: [usize; 5] = [0x12, 0x21, 0x48, 0x63, 0x6A];

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FIRST_RVA: usize = rva(0x755944); // dword_755944
const COUNTER_RVA: usize = rva(0x755950); // dword_755950
const ENDED_RVA: usize = rva(0x755928); // dword_755928
const EXTRA: [usize; 0] = [];
const CASES: u32 = 3;

fn test_entry(_case: u32) -> Regs {
    Regs::SENTINEL
}

fn prepare(case: u32) {
    unsafe {
        game_write(INDEX_RVA, case);
        game_write(crate::math::next_random::STATE_RVA, 0x2468_ACE0u32 ^ case);
    }
}

/// The record at `index` of the current section.
fn record_at(index: u32) -> u32 {
    let table = game_addr(RECORDS_RVA) as u32;
    let section: u32 = unsafe { game_read(SECTION_RVA) };
    crate::records::find_record::find_record_in(table, index as i32, section)
}

/// The records the walk visits from `index`, with their steps, checked.
fn walk() -> Result<Vec<(u32, Step)>, Refused> {
    current_event()?;
    counted_players()?;
    entries(game_addr(DRIVERS_RVA) as u32, DRIVER)?;
    let index: u32 = unsafe { game_read(INDEX_RVA) };
    // Converted: `rc2k_core::menu::section_walk::walk`.
    rc2k_core::menu::section_walk::walk(
        index,
        |i| get(record_at(i)),
        |i| crate::menu::calc_cpu_times::check(record_at(i)).map(|_| ()),
    )
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn walk_section_records(entry: &Regs) -> Result<Regs, Refused> {
    use crate::menu::{reset_driver_condition as rdc, reset_player_totals as rpt};
    let records = walk()?;
    unsafe {
        for (_, step) in records {
            if !step.goes_on() {
                if step == Step::Ended {
                    game_write(ENDED_RVA, 1u32);
                }
                break;
            }
            match step {
                Step::CpuTimes => {
                    crate::menu::calc_cpu_times::calc_cpu_times(entry)?;
                }
                Step::Condition => {
                    rdc::reset_driver_condition(entry)?;
                }
                _ => {}
            }
            if let Some(first) = step.first() {
                game_write(FIRST_RVA, first);
            }
            game_write(INDEX_RVA, game_read::<u32>(INDEX_RVA).wrapping_add(1));
            game_write(COUNTER_RVA, game_read::<u32>(COUNTER_RVA).wrapping_add(1));
            rpt::reset_player_totals(entry)?;
        }
        rdc::reset_driver_condition(entry)?;
    }
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    let first = game_addr(DRIVERS_RVA) as u32;
    let Ok(drivers) = entries(first, DRIVER) else {
        return vec![(0, 0)];
    };
    let mut v = vec![
        (first, drivers.len() * DRIVER as usize),
        (game_addr(rva(0x75AAE8)) as u32, 0x18),
        (game_addr(rva(0x71C3D8)) as u32, 8),
        at(rva(0x7559EC)),
        at(rva(0x7559F0)),
        at(rva(0x755948)),
        at(crate::math::next_random::STATE_RVA),
        at(FIRST_RVA),
        at(COUNTER_RVA),
        at(ENDED_RVA),
        at(INDEX_RVA),
    ];
    v.extend(
        (0..4u32)
            .map(|n| (player(n), 0x400))
            .filter(|&(p, _)| p != 0),
    );
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { walk_section_records(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    // It logs: in shadow mode the original alone (the self-test compares the two).
    if crate::framework::shadow() {
        HOOK.count_call();
        unsafe { call_original(r) };
        return;
    }
    let regions = regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    // Unpaused (it logs): the generator to this test alone meanwhile.
    crate::math::next_random::own_random(test)
}

fn test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Unpaused: frames drawn meanwhile may see the tables this test changes.
    let _disturbing = crate::screens::harness::Disturbing::begin();
    let regions = regions();
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { walk_section_records(&test_entry(0)) }.is_err();
    restore(&vars, &saved);
    if refused {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        // How many numbers each side draws (`next_random`'s count; the original's draws
        // go through its hook too). With shadow off only: in shadow mode the hooks run both
        // versions of every draw.
        if !crate::framework::shadow() {
            use crate::math::next_random::DRAWS;
            use std::sync::atomic::Ordering;
            restore(&vars, &saved);
            prepare(case);
            DRAWS.store(0, Ordering::SeqCst);
            let mut ro = test_entry(case);
            unsafe { call_original(&mut ro) };
            let theirs = DRAWS.load(Ordering::SeqCst);
            restore(&vars, &saved);
            prepare(case);
            DRAWS.store(0, Ordering::SeqCst);
            let _ = unsafe { walk_section_records(&test_entry(case)) };
            let mine = DRAWS.load(Ordering::SeqCst);
            t.check(theirs == mine, || {
                format!("case {case}: random numbers drawn: original {theirs}, rust {mine}")
            });
        }
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            test_entry(case),
            &|| {
                restore(&vars, &saved);
                prepare(case);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
