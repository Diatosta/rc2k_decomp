//! `net::send_turn` (0x46415B): the own car's messages by its step count (the current
//! car's block +8, `unk_719FE0`): every fourth step (& 3 = 0) `net::send_inputs`, at & 0x1F
//! = 2 `net::send_position`, at & 0xFF = 1 `net::send_extra`.
//!
//! In:  nothing
//! Out: eax the count & 0xFF, or as `send_extra` leaves the registers; the other
//!      registers as the last send made leaves them
//!
//! See `net/mod.rs`. One caller, `net::race_step`. Its calls through `part`. Shadow mode
//! runs the original alone. Self-test: a copy whose count is the test's and whose calls go
//! to noting stand-ins, against the Rust version, on counts that make each send.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::{CAR, Part, SEND_EXTRA, SEND_INPUTS, SEND_POSITION, a, dispatch_net, net_entry};
use crate::system::api_stand_ins::copy_with;
use rc2k_core::net::race::SENT;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::send_turn", rva(0x46415B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version, the count read at `steps`.
pub fn send_turn_with(r: &mut Regs, steps: u32, part: Part) -> Result<(), Refused> {
    let count = || unsafe { (steps as *const u32).read_volatile() };
    // Converted: `rc2k_core::net::race::Kind::due`.
    for (kind, place) in SENT
        .into_iter()
        .zip([SEND_INPUTS, SEND_POSITION, SEND_EXTRA])
    {
        let (due, masked) = kind.due(count());
        r.eax = masked;
        if due {
            part(place, r)?;
        }
    }
    Ok(())
}

/// The Rust version.
pub fn send_turn(r: &mut Regs, part: Part) -> Result<(), Refused> {
    send_turn_with(r, a(CAR) + 8, part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, send_turn);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, send_turn)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut steps = 0u32;
    let at = &raw mut steps as u32;
    // mov eax, dword ptr unk_719FE0
    let entry = [0xA1, 0xE0, 0x9F, 0x71, 0x00];
    let copy = match copy_with(
        0x46415B,
        0x39,
        &entry,
        &[0xD, 0x1F, 0x33],
        &[SEND_INPUTS, SEND_POSITION, SEND_EXTRA],
        &[],
        &[(CAR + 8, 4, at)],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for count in [0u32, 1, 2, 3, 4, 0x22, 0x101, 0x102, 0x100, 0xFFFF_FFFF] {
        compare(
            &mut t,
            &format!("count {count:#x}"),
            &copy,
            Regs::SENTINEL,
            &|| unsafe { (at as *mut u32).write_volatile(count) },
            &[(at, 4)],
            &|r, part| send_turn_with(r, at, part),
            false,
        );
    }
    t.finish()
}
