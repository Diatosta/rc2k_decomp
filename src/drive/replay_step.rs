//! `drive::replay_step` (0x460B2F): the two-player / replay race's step (`sub_4455A9`'s).
//! `drive::choose_shown_cars`; the replay's frame kept (`drive::keep_replay_frame`) and,
//! playing, read (`drive::play_replay_frame`); the skid marks begun again
//! (`race::init_skidmarks`) unless the first block's clock (+8) is one past +0xF0C, which
//! takes it. For each car (`g_currentCar` from 0, in `pusha` / `popa`): its input words
//! (`drive::car_input_words`, recorded or played: `record_inputs_if`, `play_inputs_if`),
//! its block into `dword_71B404`, `drive::load_car`; running (`unk_71A234`): controls,
//! gear rules, steering, pedals, the reset key, `physics::car_physics_step` and the
//! network's step (`sub_464133`, DirectPlay: a call into the game); the work car's clocks
//! (`unk_719FE0`, `unk_719FEC`) + 1; `drive::store_car`. Then the stage's animation, the
//! race-mode cars, the duel's sound levels, the count down's beeps, the two cars' order,
//! the hold count, the replay written at its last tenth, the replay's clock.
//!
//! In:  nothing
//! Out: the registers as the last steps leave them; no flags read
//!
//! Caller: `drive::race_step`'s `sub_4455A9`. It begins with a call: hooked by a private
//! copy of its 0xE1 bytes (its 25 calls relocated) and a hand-made patch.
//! Refuses (before anything) a cars' count of 0 or past 8, and what its calls refuse
//! (the cars' blocks, the work car and the physics' variables put back). Shadow mode runs
//! the original alone (as `car_physics_step`).
//! Self-test (in a race; skipped otherwise): the original and the Rust version from the
//! same state, logging off, no replay (`dword_5C5CE0` 0).

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, ESI, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow,
};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::flow::{g, p};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("drive::replay_step", rva(0x460B2F), thunk, &ORIGINAL)
    .private_copy(0xE1)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

#[rustfmt::skip]
const CALLS: [usize; 25] = [
    0x0, 0x5, 0xA, 0x1F, 0x39, 0x3E, 0x43, 0x5D, 0x6B, 0x70, 0x75, 0x7A, 0x7F, 0x84, 0x89,
    0x9A, 0xB3, 0xB8, 0xBD, 0xC2, 0xC7, 0xCC, 0xD1, 0xD6, 0xDB,
];

const B: usize = 0x70F3EC; // g_carBlockStore
const CURRENT: usize = 0x71B3F8;
const CARS: usize = 0x63DB90;
/// `sub_464133`: the network's step (DirectPlay), the game's.
const NETWORK: usize = 0x464133;

fn car_step(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        crate::drive::car_input_words::car_input_words(r)?.apply(r);
        crate::drive::record_inputs_if::record_inputs_if().apply(r);
        crate::drive::play_inputs_if::play_inputs_if(r).apply(r);
        r.eax = g(CURRENT) & 7;
        r.esi = g(0x71B3B8 + 4 * r.eax as usize);
        p(0x71B404, r.esi);
        crate::drive::load_car::load_car()?.apply(r);
        if g(0x71A234) != 0 {
            let mut c = *r;
            crate::drive::read_car_controls::read_car_controls(&mut c)?;
            Left::nothing().with(EAX, c.eax).with(ESI, c.esi).apply(r);
            let mut c = *r;
            crate::physics::gear_input_rules::gear_input_rules(&mut c);
            r.eax = c.eax;
            crate::drive::steer_inputs::steer_inputs(r).apply(r);
            crate::drive::pedal_inputs::pedal_inputs().apply(r);
            *r = crate::drive::reset_request::reset_request(
                r,
                &mut crate::physics::reset_parts::game_part,
            )?;
            crate::physics::car_physics_step::car_physics_step(r)?.apply(r);
            call_game(r, game_addr(rva(NETWORK)));
        }
        p(0x719FE0, g(0x719FE0).wrapping_add(1));
        p(0x719FEC, g(0x719FEC).wrapping_add(1));
        crate::drive::store_car::store_car()?.apply(r);
    }
    Ok(())
}

/// The steps; `r` as the original leaves it.
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        crate::drive::choose_shown_cars::choose_shown_cars()?.apply(r);
        crate::drive::keep_replay_frame::keep_replay_frame().apply(r);
        crate::drive::play_replay_frame::play_replay_frame(r)?.apply(r);
    }
    r.eax = g(B + 8).wrapping_sub(g(B + 0xF0C));
    if r.eax != 1 {
        *r = unsafe { crate::race::stage_prefs::init_skidmarks(r) };
    }
    r.eax = g(B + 8);
    p(B + 0xF0C, r.eax);
    p(CURRENT, 0);
    loop {
        let mut c = *r;
        car_step(&mut c)?;
        p(CURRENT, g(CURRENT).wrapping_add(1));
        r.eax = g(CURRENT);
        if r.eax == g(CARS) {
            break;
        }
    }
    unsafe {
        crate::drive::animate_stage::animate_stage(r)?;
        *r = crate::drive::race_mode_cars::race_mode_cars(r, &mut |_, r| {
            *r = crate::drive::start_cars_behind::start_cars_behind(
                r,
                &mut crate::drive::start_cars_behind::game_part,
            )?;
            Ok(())
        })?;
        crate::drive::duel_sound_levels::duel_sound_levels(r).apply(r);
        crate::drive::countdown_beeps::countdown_beeps(r)?.apply(r);
        crate::drive::two_car_order::two_car_order(r).apply(r);
        crate::drive::hold_count_tick::hold_count_tick();
        crate::drive::flush_replay_if::flush_replay_if(r)?.apply(r);
        crate::drive::replay_clock_tick::replay_clock_tick();
    }
    Ok(())
}

/// The Rust version.
pub unsafe fn replay_step(entry: &Regs) -> Result<Left, Refused> {
    if !(1..=8).contains(&g(CARS)) {
        return Err(BadInput("a cars' count of 0 or past 8").into());
    }
    let all = crate::drive::race_cars_step::cars_regions();
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
    match unsafe { replay_step(r) } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !crate::physics::move_slot::in_race() || !(1..=8).contains(&g(CARS)) {
        return t.skip("not in a race (no stage)");
    }
    let mut all = crate::drive::race_cars_step::cars_regions();
    all.extend(crate::physics::car_physics_step::test_regions());
    let a = |x: usize| game_addr(rva(x)) as u32;
    all.extend([
        (a(0x4E0094), 4),
        (a(0x5C5CE0), 4),
        (a(0x5C2530), 8),
        (a(0x6EF7D8), 8),
        (a(0x6235EC), 4),
        (a(0x6D1482), 0x14),
        (a(0x614114), 4),
        (a(0x5165A8), 0x1C),
        (a(0x6138C4), 0x10),
    ]);
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            let set = || {
                restore(&all, &saved);
                p(0x4E0094, 0);
                p(0x5C5CE0, 0);
            };
            set();
            let mut ro = Regs::SENTINEL;
            unsafe { call_original(&mut ro) };
            let theirs = (left_of(&ro), capture(&all));
            set();
            let mine = unsafe { replay_step(&Regs::SENTINEL) };
            let state = capture(&all);
            t.check(mine.as_ref().is_ok_and(|l| *l == theirs.0) && state == theirs.1, || {
                let diff = theirs.1.iter().zip(&state).position(|(x, y)| x != y);
                format!("control word {word:#x}: original {:x?}; rust {mine:x?}; memory first differs at {diff:x?}", theirs.0)
            });
        });
    }
    restore(&all, &saved);
    t.finish()
}
