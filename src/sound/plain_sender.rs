//! The game's smallest command senders (stream B, 2026-10-18): one `push n; call
//! sound::send_command` with the registers as they came, and no wait (unlike
//! `sound/senders.rs`'s). What their hooks and self-tests share.
//!
//! Shadow mode runs each original alone (a command must not be sent twice). Self-tests:
//! a copy whose calls go to stand-ins (the one of `send_command` reads the command off
//! the stack), against the Rust version with a part that notes the same.

use crate::framework::{Regs, SelfTest, call_game, game_addr, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::CallOriginal;
use crate::sound::queue::{Queue, send_command, sent};
use crate::sound::turn_parts::regs7;
use std::sync::Mutex;

/// `sound::send_command`.
pub const SEND: usize = 0x409C45;

/// What a sender's Rust version does: (callee, command or 0, registers), in order.
pub type Calls = Vec<(usize, u32, [u32; 7])>;

/// A sender's Rust version: it calls `call(callee, command, registers)` for each call.
pub type Sender = fn(&Regs, &mut dyn FnMut(usize, u32, &Regs));

/// A command sent in the game with the registers of `r`.
pub fn send_in_game(command: u32, r: &Regs) {
    unsafe { send_command(&Queue::game(), command, sent(r)) };
}

/// The hooks' dispatch: the original alone in shadow mode, else `rust` in the game.
pub fn dispatch_plain(
    hook: &crate::framework::RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    rust: Sender,
) {
    hook.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    rust(r, &mut |callee, command, regs| match callee {
        SEND => send_in_game(command, regs),
        0x409BE0 => crate::sound::send_end_of_turn::send_end_of_turn_in_game(regs),
        // sound::channels_in_use: flags only, which no one reads here.
        _ => {}
    });
}

/// `dispatch_plain` without its hook (no shadow mode; nothing refused).
pub fn plain_entry(r: &mut Regs, rust: Sender) -> Result<(), crate::errors::Refused> {
    rust(r, &mut |callee, command, regs| match callee {
        SEND => send_in_game(command, regs),
        0x409BE0 => crate::sound::send_end_of_turn::send_end_of_turn_in_game(regs),
        // sound::channels_in_use: flags only, which no one reads here.
        _ => {}
    });
    Ok(())
}

static CALLED: Mutex<(Vec<usize>, Calls)> = Mutex::new((Vec::new(), Vec::new()));

fn handler(n: u32, r: &mut Regs) {
    let mut c = CALLED.lock().unwrap();
    let callee = c.0[n as usize];
    // Above the stand-in's flags: the return address, then the command.
    let command = if callee == SEND {
        unsafe { ((r.esp as usize + 8) as *const u32).read_unaligned() }
    } else {
        0
    };
    c.1.push((callee, command, regs7(r)));
}

/// The self-test of one sender: the copy (`len` bytes, first bytes `entry`, calls at
/// `calls` to `callees`) against `rust`, for two sets of registers.
pub fn test_plain(
    t: &mut SelfTest,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    callees: &[usize],
    rust: Sender,
) {
    set_handler(handler);
    CALLED.lock().unwrap().0 = callees.to_vec();
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|c| {
            callees
                .iter()
                .position(|&k| rva(k) == c)
                .map(|n| stand_in(n as u32))
        })
    };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => {
            t.check(false, || format!("no copy: {why}"));
            return;
        }
    };
    for eax in [0u32, 0x1234_5678] {
        let entry = Regs {
            eax,
            ..Regs::SENTINEL
        };
        CALLED.lock().unwrap().1.clear();
        let mut o = entry;
        unsafe { call_game(&mut o, copy.addr()) };
        let theirs = (regs7(&o), std::mem::take(&mut CALLED.lock().unwrap().1));
        let mut mine_calls: Calls = Vec::new();
        rust(&entry, &mut |callee, command, r| {
            mine_calls.push((callee, command, regs7(r)))
        });
        let mine = (regs7(&entry), mine_calls);
        t.check(theirs == mine, || {
            format!("eax {eax:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
}
