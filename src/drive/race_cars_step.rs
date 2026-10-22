//! `drive::race_cars_step` (0x4608BB): every car's physics step. `drive::choose_shown_cars`;
//! for each car n of `g_raceCars` (`g_currentCar` = n, from 0): in `pusha` / `popa`, the
//! four `dword_6138C4` .. 0, its block (`g_carBlocks[n & 7]`) into `dword_71B404`,
//! `drive::load_car`; with the car running (`unk_71A234` set) `drive::read_car_controls`,
//! `physics::gear_input_rules` and `physics::car_physics_step`; `drive::store_car`. Then
//! `drive::animate_stage` and `drive::race_mode_cars`.
//!
//! In:  nothing
//! Out: eax the cars' count, then as `animate_stage` and `race_mode_cars` leave the
//!      registers; no flags read
//!
//! Caller: `drive::race_init` (its fifty settling steps). It begins with a call: hooked by
//! a private copy of its 0x8E bytes with its eight calls relocated, and a hand-made
//! patch.
//! Refuses (before anything) a cars' count of 0 or past 8 (the original's `jnz` loop runs
//! on), and what its calls refuse; the cars' blocks, the work car and the physics'
//! variables are put back then. Shadow mode runs the original alone (as
//! `car_physics_step`).
//! Self-test (in a race; skipped otherwise): the original and the Rust version from the
//! same state (everything written saved and put back), logging off. The pace notes'
//! sound cues (`physics::pace_notes_step` -> the sound objects' queues) are not put back
//! between the two runs: a mismatch there would show in those queues only.

use crate::errors::Refused;
use crate::framework::{EAX, ESI, RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::flow::{g, p};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, restore};
use rc2k_core::drive::race_steps;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("drive::race_cars_step", rva(0x4608BB), thunk, &ORIGINAL)
    .private_copy(0x8E)
    .relocate_calls(&[0x0, 0x4D, 0x5B, 0x60, 0x65, 0x6A, 0x83, 0x88])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CURRENT: usize = 0x71B3F8; // g_currentCar
const CARS: usize = 0x63DB90; // g_raceCars
const BLOCKS: usize = 0x71B3B8; // g_carBlocks

/// One car's step (0x4608CA .., inside `pusha` / `popa`).
fn car_step(r: &mut Regs) -> Result<(), Refused> {
    for x in [0x6138C4, 0x6138C8, 0x6138CC, 0x6138D0] {
        p(x, 0);
    }
    r.eax = race_steps::block_slot(g(CURRENT)) as u32;
    r.esi = g(BLOCKS + 4 * r.eax as usize);
    p(0x71B404, r.esi);
    unsafe {
        crate::drive::load_car::load_car()?.apply(r);
        if g(0x71A234) != 0 {
            let mut c = *r;
            crate::drive::read_car_controls::read_car_controls(&mut c)?;
            Left::nothing().with(EAX, c.eax).with(ESI, c.esi).apply(r);
            let mut c = *r;
            crate::physics::gear_input_rules::gear_input_rules(&mut c);
            r.eax = c.eax;
            crate::physics::car_physics_step::car_physics_step(r)?.apply(r);
        }
        crate::drive::store_car::store_car()?.apply(r);
    }
    Ok(())
}

/// The steps; `r` as the original leaves it.
// Converted: `rc2k_core::drive::race_steps::{check_cars, each_car, block_slot}`.
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::drive::choose_shown_cars::choose_shown_cars() }?.apply(r);
    r.eax = race_steps::each_car(
        || g(CURRENT),
        || g(CARS),
        |n| p(CURRENT, n),
        || car_step(&mut r.clone()),
    )?;
    unsafe {
        crate::drive::animate_stage::animate_stage(r)?;
        *r = crate::drive::race_mode_cars::race_mode_cars(r, &mut |_, r| {
            *r = crate::drive::start_cars_behind::start_cars_behind(
                r,
                &mut crate::drive::start_cars_behind::game_part,
            )?;
            Ok(())
        })?;
    }
    Ok(())
}

/// The cars' blocks, the step's own words and the physics' (put back on a refusal).
pub fn cars_regions() -> Vec<Region> {
    let mut all = crate::physics::car_physics_step::step_regions();
    for k in 0..8 {
        let block = g(BLOCKS + 4 * k);
        if block != 0 {
            all.push((block, 0xFE0));
        }
    }
    let a = |x: usize| game_addr(rva(x)) as u32;
    all.extend([(a(0x6138C4), 0x10), (a(0x71B404), 4), (a(CURRENT), 4)]);
    all
}

/// The Rust version.
pub unsafe fn race_cars_step(entry: &Regs) -> Result<Left, Refused> {
    race_steps::check_cars(g(CARS))?;
    let all = cars_regions();
    let saved = capture(&all);
    let mut r = *entry;
    match steps(&mut r) {
        Ok(()) => Ok(left_of(&r)),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { race_cars_step(r) } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { race_cars_step(r) }?;
    left.apply(r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !crate::physics::move_slot::in_race() || !(1..=8).contains(&g(CARS)) {
        return t.skip("not in a race (no stage)");
    }
    let mut all = cars_regions();
    all.extend(crate::physics::car_physics_step::test_regions());
    all.push((game_addr(rva(0x4E0094)) as u32, 4));
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            let set = || {
                restore(&all, &saved);
                p(0x4E0094, 0);
            };
            set();
            let mut ro = Regs::SENTINEL;
            unsafe { call_original(&mut ro) };
            let theirs = (left_of(&ro), capture(&all));
            set();
            let mine = unsafe { race_cars_step(&Regs::SENTINEL) };
            let state = capture(&all);
            t.check(mine.as_ref().is_ok_and(|l| *l == theirs.0) && state == theirs.1, || {
                let diff = theirs.1.iter().zip(&state).position(|(x, y)| x != y);
                format!(
                    "control word {word:#x}: original {:x?}; rust {mine:x?}; memory first differs at {diff:x?}",
                    theirs.0
                )
            });
        });
    }
    restore(&all, &saved);
    t.finish()
}
