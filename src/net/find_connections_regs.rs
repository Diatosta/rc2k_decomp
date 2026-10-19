//! `net::find_connections_regs` (0x4174BD): `net::find_connections` inside `pusha` .. `popa`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the carry `net::find_connections` leaves (set: no connection)
//! Callers: `net::start_link`, `net::settings_link`, `net::settings_done`. Its call through `part`. Shadow mode runs the original alone. Self-test: a copy whose call goes to a
//! stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{FLAGS, Part, answer, dispatch_lobby, run_both, test_eax};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "net::find_connections_regs",
    rva(0x4174BD),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CALLEE: u32 = 0x41867C; // net::find_connections

/// The Rust version: the registers it leaves (the entry's, the flags the call's).
pub fn find_connections_regs(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(CALLEE, &mut r)?;
    let _ = test_eax;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, find_connections_regs);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match crate::net::lobby::copy(
        0x4174BD,
        0x8,
        &[0x60, 0xE8, 0xB9, 0x11, 0x00, 0x00],
        &[0x1],
        &[CALLEE],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };

    for (eax, carry) in [(0u32, false), (5, true), (0x8000_0000, false)] {
        run_both(
            &mut t,
            &format!("eax {eax:#x}, the call's carry {carry}"),
            copy.addr(),
            Regs {
                eax,
                ..Regs::SENTINEL
            },
            &[],
            FLAGS,
            &[],
            &|| {
                answer(Box::new(move |_, r| {
                    r.eax = 0x77;
                    r.ebx = 0x66;
                    r.set_carry(carry);
                    r.set_zero(!carry);
                }))
            },
            &find_connections_regs,
        );
    }

    t.finish()
}
