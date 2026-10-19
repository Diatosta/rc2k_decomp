//! `net::settings_done` (0x416F0B): the connection saved checked: the connections listed
//! (`net::find_connections_regs`) and the saved one among them
//! (`net::find_saved_connection`) kept again (`net::keep_connection`, its name fresh);
//! if either fails the saved connection forgotten (`unk_52AC94` 16 bytes, `unk_52ACA4` 64,
//! `system::zero_memory`); then the network stopped (`net::stop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::settings_loop` (on entering it). Its calls through `part`. It waits for the
//! receiving thread: ungated. Shadow mode runs the original alone. Self-test: a copy whose
//! calls go to stand-ins, against the Rust version: found, not found, none listed.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    Part, SAVED_CONNECTION, SAVED_NAME, STOP, ZERO_MEMORY, a, answer, dispatch_lobby, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::settings_done", rva(0x416F0B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const FIND_CONNECTIONS: u32 = 0x4174BD; // net::find_connections_regs
const FIND_SAVED: u32 = 0x4185DF; // net::find_saved_connection
const KEEP_CONNECTION: u32 = 0x417661; // net::keep_connection

/// The Rust version: the registers it leaves (the entry's).
pub fn settings_done(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(FIND_CONNECTIONS, &mut r)?;
    let mut found = !r.carry();
    if found {
        part(FIND_SAVED, &mut r)?;
        found = !r.carry();
        if found {
            part(KEEP_CONNECTION, &mut r)?;
        }
    }
    if !found {
        for (at, len) in [(SAVED_CONNECTION, 0x10), (SAVED_NAME, 0x40)] {
            let mut c = Regs {
                ebx: a(at),
                ecx: len,
                ..r
            };
            part(ZERO_MEMORY, &mut c)?;
            r.eflags = c.eflags;
        }
    }
    part(STOP, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, settings_done);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call sub_4174BD
    let copy = match crate::net::lobby::copy(
        0x416F0B,
        0x45,
        &[0x60, 0xE8, 0xAC, 0x05, 0x00, 0x00],
        &[0x1, 0x8, 0xF, 0x14, 0x28, 0x3C],
        &[
            FIND_CONNECTIONS,
            FIND_SAVED,
            KEEP_CONNECTION,
            STOP,
            ZERO_MEMORY,
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    for (none, not_found) in [(false, false), (false, true), (true, false)] {
        run_both(
            &mut t,
            &format!("none listed {none}, not found {not_found}"),
            copy.addr(),
            Regs::SENTINEL,
            &[],
            0,
            &[],
            &|| {
                answer(Box::new(move |place, r| match place {
                    FIND_CONNECTIONS => r.set_carry(none),
                    FIND_SAVED => {
                        r.set_carry(not_found);
                        r.eax = 1;
                    }
                    _ => {}
                }))
            },
            &settings_done,
        );
    }
    t.finish()
}
