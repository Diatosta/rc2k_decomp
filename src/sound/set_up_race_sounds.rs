//! `sound::set_up_race_sounds` (0x40F02E): puts the audio thread in its mode for a race.
//!
//! In:  nothing (the other registers go into the commands as they are)
//! Out: every register preserved
//! `dword_518080` = 2; `sound::stop_music`; command 5 with the race's sound mode
//! `dword_4F82C8`; `sound::load_race_sounds`; `sound::wait_for_commands`; `dword_506C84`
//! = 0 (`sub_409B76`). The menus' counterpart is `sound::apply_settings`.
//! Nine callers (the events' starters, before a stage is loaded).
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose four calls note the registers they get.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{at, capture, restore};
use crate::sound::apply_settings::{Step, run_steps};
use crate::sound::queue::{Queue, send_command, sent};
use crate::sound::stop_music::stop_music_in_game;
use crate::sound::wait_for_commands::{Game, wait_for_commands};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::set_up_race_sounds", rva(0x40F02E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const APPLIED_RVA: usize = rva(0x518080); // dword_518080
const HELD_BACK_RVA: usize = rva(0x506C84); // dword_506C84
const RACE_MODE_RVA: usize = rva(0x4F82C8); // dword_4F82C8
const LOAD_RVA: usize = rva(0x40EED8); // sound::load_race_sounds

// ---- Rust version ----

pub unsafe fn set_up_race_sounds(entry: &Regs, step: &mut dyn FnMut(Step, &Regs)) {
    // Converted: `rc2k_core::sound::set_up::race`.
    use rc2k_core::sound::set_up::{RACE, race};
    unsafe {
        game_write(APPLIED_RVA, RACE);
        run_steps(entry, &race(game_read(RACE_MODE_RVA)), step);
        game_write(HELD_BACK_RVA, 0u32);
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe {
        set_up_race_sounds(r, &mut |step, regs| match step {
            Step::StopMusic => stop_music_in_game(regs),
            Step::Send { command, .. } => send_command(&Queue::game(), command, sent(regs)),
            // Through its hook, which has the way out for what the Rust version refuses.
            Step::LoadRaceSounds => call_game(&mut { *regs }, game_addr(LOAD_RVA)),
            Step::Wait => wait_for_commands(&mut Game(sent(regs))),
        });
    }
}

// ---- self-test ----

const LEN: usize = 0x2B;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xC7, 0x05, 0x80, 0x80, 0x51, 0x00, 0x02];
const CALLS: [usize; 5] = [0x0B, 0x15, 0x1A, 0x1F, 0x24];
/// `sound::stop_music`, the sender of command 5, `sound::load_race_sounds`,
/// `sound::wait_for_commands`; the fifth callee, `sub_409B76`, stores its 0 itself.
const CALLEES: [usize; 4] = [0x40F826, 0x409BD5, 0x40EED8, 0x409C1E];

/// Each call: its place and the seven registers a command holds.
static CALLED: Mutex<Vec<(u32, [u32; 7])>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((place, sent(r)));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x40F02E)),
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
    let regions = [at(APPLIED_RVA), at(HELD_BACK_RVA)];
    let inputs = [at(RACE_MODE_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    for mode in [0u32, 0x11221, 0x0244_1220, 0xFFFF_FFFF] {
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            game_write(RACE_MODE_RVA, mode);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let preserved = if rust {
                set_up_race_sounds(&before, &mut |step, regs| {
                    let place = match step {
                        Step::StopMusic => 0,
                        Step::Send { .. } => 1,
                        Step::LoadRaceSounds => 2,
                        Step::Wait => 3,
                    };
                    act(place, &mut { *regs });
                });
                true
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
            };
            (preserved, capture(&regions), CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.2.len() == 4, || {
            format!("mode {mode:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
