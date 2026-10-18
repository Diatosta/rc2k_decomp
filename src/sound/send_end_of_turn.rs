//! `sound::send_end_of_turn` (0x409BE0): tells the audio thread a turn is over
//! (command 0xA).
//!
//! In:  the registers go into the command as they are
//! Out: every register preserved
//! `push eax; push 0xA; call sound::send_command; pop eax; pop eax`. Callers:
//! `sound::race_turn`, `sound::wait_for_commands` (twice, whose Rust version sends it
//! itself), `sound::send_command_7`.
//! A call in its first five bytes: private copy (0xB bytes, the call relocated), manual
//! patch.
//! Shadow mode runs the original alone (a command must not be sent twice).
//! Self-test: a copy whose call goes to a stand-in that notes the registers and the
//! command on the stack, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::sound::queue::{END_OF_TURN, Queue, send_command, sent};
use crate::sound::turn_parts::{Part, dispatch_turn, regs7};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("sound::send_end_of_turn", rva(0x409BE0), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&[3])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0xB;
const ENTRY: [u8; 8] = [0x50, 0x6A, 0x0A, 0xE8, 0x5D, 0x00, 0x00, 0x00];
/// `sound::send_command`: its place, and the command it is called with here.
pub const SEND: u32 = 0x409C45;

/// Command 0xA with the registers of `r`, in the game.
pub fn send_end_of_turn_in_game(r: &Regs) {
    unsafe { send_command(&Queue::game(), END_OF_TURN, sent(r)) };
}

// ---- Rust version ----

pub fn send_end_of_turn_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    part(SEND, r)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, send_end_of_turn_in, None);
}

// ---- self-test ----

static CALLED: Mutex<Vec<([u32; 7], u32)>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    // Above the stand-in's flags: the return address, then the command.
    let command = unsafe { ((r.esp as usize + 8) as *const u32).read_unaligned() };
    CALLED.lock().unwrap().push((regs7(r), command));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    set_handler(handler);
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x409BE0)), LEN, &ENTRY, &[3], &|c| {
            (c == rva(SEND as usize)).then(|| stand_in(0))
        })
    };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    for eax in [0u32, 0x1234_5678] {
        let entry = Regs {
            eax,
            ..Regs::SENTINEL
        };
        CALLED.lock().unwrap().clear();
        let mut o = entry;
        unsafe { crate::framework::call_game(&mut o, copy.addr()) };
        let theirs = (regs7(&o), std::mem::take(&mut *CALLED.lock().unwrap()));
        let mut g = entry;
        let mut mine_calls = Vec::new();
        let ok = send_end_of_turn_in(&mut g, &mut |place, r| {
            mine_calls.push((regs7(r), if place == SEND { END_OF_TURN } else { 0 }));
            Ok(())
        })
        .is_ok();
        let mine = (regs7(&g), mine_calls);
        t.check(ok && theirs == mine, || {
            format!("eax {eax:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    t.finish()
}
