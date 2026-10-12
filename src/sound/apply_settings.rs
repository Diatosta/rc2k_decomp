//! `sound::apply_settings` (0x40EFF3): sends the kept sound settings to the audio thread.
//!
//! In:  nothing (the other registers go into the commands as they are)
//! Out: every register preserved
//! `dword_518080` = 1; `sound::stop_music`; command 5 with `dword_4F82C4`; command 3 with the
//! address of `unk_511AE4`; command 8 with `dword_755684`; `sound::wait_for_commands`;
//! `dword_506C84` = 0 (`sub_409B76`).
//! What the two numbers are is a reading: volumes, the second the one the options screen
//! steps between 0 and 0x40 (`sub_467F00`, `sub_467F24`).
//! Five callers; `system::main_loop` calls it once the window is there.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose six calls note the registers they get.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{at, capture, restore};
use crate::sound::queue::{Queue, SET_5, SET_8, START_3, send_command, sent};
use crate::sound::stop_music::stop_music_in_game;
use crate::sound::wait_for_commands::{Game, wait_for_commands};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::apply_settings", rva(0x40EFF3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const APPLIED_RVA: usize = rva(0x518080); // dword_518080
const CLEARED_RVA: usize = rva(0x506C84); // dword_506C84
const FIRST_RVA: usize = rva(0x4F82C4); // dword_4F82C4
const THIRD_RVA: usize = rva(0x511AE4); // unk_511AE4
const SECOND_RVA: usize = rva(0x755684); // dword_755684

// ---- Rust version ----

/// What the function does besides its two stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Silence,
    /// A command, queued with these registers.
    Send(u32),
    Wait,
}

pub unsafe fn apply_settings(entry: &Regs, step: &mut dyn FnMut(Step, &Regs)) {
    unsafe {
        game_write(APPLIED_RVA, 1u32);
        step(Step::Silence, entry);
        let with = |eax: u32| Regs { eax, ..*entry };
        let r = with(game_read(FIRST_RVA));
        step(Step::Send(SET_5), &r);
        let r = with(game_addr(THIRD_RVA) as u32);
        step(Step::Send(START_3), &r);
        let r = with(game_read(SECOND_RVA));
        step(Step::Send(SET_8), &r);
        step(Step::Wait, &r);
        game_write(CLEARED_RVA, 0u32);
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
        apply_settings(r, &mut |step, regs| match step {
            Step::Silence => stop_music_in_game(regs),
            Step::Send(command) => send_command(&Queue::game(), command, sent(regs)),
            Step::Wait => wait_for_commands(&mut Game(sent(regs))),
        });
    }
}

// ---- self-test ----

const LEN: usize = 0x3B;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xC7, 0x05, 0x80, 0x80, 0x51, 0x00, 0x01];
const CALLS: [usize; 6] = [0x0B, 0x15, 0x20, 0x2A, 0x2F, 0x34];
/// `sound::stop_music`, the senders of commands 5, 3 and 8, `sound::wait_for_commands`; the
/// sixth callee, `sub_409B76`, stores its 0 itself.
const CALLEES: [usize; 5] = [0x40F826, 0x409BD5, 0x409BBF, 0x409BB4, 0x409C1E];

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
            game_addr(rva(0x40EFF3)),
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
    let regions = [at(APPLIED_RVA), at(CLEARED_RVA)];
    let inputs = [at(FIRST_RVA), at(SECOND_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    for (first, second) in [(0u32, 0u32), (0x40, 0x20), (0xFFFF_FFFF, 0x8000_0000)] {
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            game_write(FIRST_RVA, first);
            game_write(SECOND_RVA, second);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let preserved = if rust {
                apply_settings(&before, &mut |step, regs| {
                    let place = match step {
                        Step::Silence => 0,
                        Step::Send(5) => 1,
                        Step::Send(3) => 2,
                        Step::Send(_) => 3,
                        Step::Wait => 4,
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
        t.check(theirs == mine && theirs.0 && theirs.2.len() == 5, || {
            format!("settings {first:#x} {second:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
