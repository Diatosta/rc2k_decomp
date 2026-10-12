//! `sound::stop_music` (0x40F826): stops the music: command 0xB with four zeros, and the
//! wait for it.
//!
//! In:  nothing (esi, edi and ebp go into the command with the zeros)
//! Out: every register preserved
//! eax, ebx, ecx and edx = 0; command 0xB (`sub_409BF6`: the audio thread's `sub_40C4DD`
//! with those four); `sound::wait_for_commands`. Command 0xB is the music's: eax a tune,
//! ebx a second one, ecx the volume, edx what to do (2 in `sound::play_music`, 3 to
//! change the volume, 0 here). That 0 stops it is a reading of its users (before films,
//! on the way out of the front end), not of the audio thread's code. Named
//! `sound::silence` at first.
//! Seventeen callers.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose two calls note the registers they get.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::sound::queue::{COMMAND_B, Queue, send_command, sent};
use crate::sound::wait_for_commands::{Game, wait_for_commands};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::stop_music", rva(0x40F826), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `entry` holds the registers at entry; `send` queues a command with the registers it
/// is given, `wait` waits with them.
pub fn stop_music(entry: &Regs, send: &mut dyn FnMut(u32, &Regs), wait: &mut dyn FnMut(&Regs)) {
    let zeroed = Regs {
        eax: 0,
        ebx: 0,
        ecx: 0,
        edx: 0,
        ..*entry
    };
    send(COMMAND_B, &zeroed);
    wait(&zeroed);
}

/// `stop_music` in the game.
pub fn stop_music_in_game(entry: &Regs) {
    stop_music(
        entry,
        &mut |command, r| unsafe { send_command(&Queue::game(), command, sent(r)) },
        &mut |r| wait_for_commands(&mut Game(sent(r))),
    );
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    stop_music_in_game(r);
}

// ---- self-test ----

const LEN: usize = 0x18;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x33, 0xC0, 0x33, 0xDB, 0x33, 0xC9, 0xBA];
/// The sender of command 0xB and `sound::wait_for_commands`.
const CALLS: [usize; 2] = [0x0C, 0x11];
const CALLEES: [usize; 2] = [0x409BF6, 0x409C1E];

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
            game_addr(rva(0x40F826)),
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
    let before = Regs::SENTINEL;
    let run = |rust: bool| {
        CALLED.lock().unwrap().clear();
        set_handler(handler);
        let preserved = if rust {
            stop_music(&before, &mut |_, r| act(0, &mut { *r }), &mut |r| {
                act(1, &mut { *r })
            });
            true
        } else {
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            ro.preserved_except(&before, 0)
        };
        (preserved, CALLED.lock().unwrap().clone())
    };
    let (theirs, mine) = (run(false), run(true));
    t.check(theirs == mine && theirs.0 && theirs.1.len() == 2, || {
        format!("original {theirs:x?}; rust {mine:x?}")
    });
    t.finish()
}
