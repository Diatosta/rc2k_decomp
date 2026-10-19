//! `net::settings_link` (0x416EEC): Enter on the settings screen's connection: the
//! connections listed (`net::find_connections_regs`), one chosen (`net::choose_connection`)
//! and kept (`net::keep_connection`); then the network stopped again (`net::stop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::settings_loop`. Its calls through `part`. Long-running (a list screen):
//! ungated. Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins, against the Rust version: no connections, none chosen (carry, a negative
//! choice), one chosen.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{Part, STOP, answer, dispatch_lobby, run_both, set_sign};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::settings_link", rva(0x416EEC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const FIND_CONNECTIONS: u32 = 0x4174BD; // net::find_connections_regs
const CHOOSE_CONNECTION: u32 = 0x4174C5; // net::choose_connection
const KEEP_CONNECTION: u32 = 0x417661; // net::keep_connection

/// The Rust version: the registers it leaves (the entry's).
pub fn settings_link(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(FIND_CONNECTIONS, &mut r)?;
    if !r.carry() {
        part(CHOOSE_CONNECTION, &mut r)?;
        if !r.carry() && (r.eax as i32) >= 0 {
            part(KEEP_CONNECTION, &mut r)?;
        }
    }
    part(STOP, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, settings_link);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call sub_4174BD
    let copy = match crate::net::lobby::copy(
        0x416EEC,
        0x1F,
        &[0x60, 0xE8, 0xCB, 0x05, 0x00, 0x00],
        &[0x1, 0x8, 0x13, 0x18],
        &[FIND_CONNECTIONS, CHOOSE_CONNECTION, KEEP_CONNECTION, STOP],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    for (none, carry, choice) in [
        (true, false, 0u32),
        (false, true, 0),
        (false, false, u32::MAX),
        (false, false, 2),
    ] {
        run_both(
            &mut t,
            &format!("none {none}, the choice's carry {carry}, choice {choice:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &[],
            0,
            &[],
            &|| {
                answer(Box::new(move |place, r| match place {
                    FIND_CONNECTIONS => r.set_carry(none),
                    CHOOSE_CONNECTION => {
                        r.eax = choice;
                        r.set_carry(carry);
                        set_sign(r, choice as i32);
                    }
                    _ => {}
                }))
            },
            &settings_link,
        );
    }
    t.finish()
}
