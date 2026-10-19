//! `net::find_saved_connection` (0x4185DF): the connection listed whose service provider's
//! GUID (record +0x40) is the one saved (`unk_52AC94`, `system::same_guid`).
//!
//! In:  nothing
//! Out: eax = its number, carry clear (the flags of `test eax, eax`); none (or none listed):
//!      eax = -1, carry set; ebx, ecx kept; edx as the calls leave it
//! Callers: `net::start_link`, `net::settings_done`. Its calls through `part`. Shadow mode
//! runs the original alone. Self-test: a copy whose calls go to stand-ins (the records
//! given, the second the same), against the Rust version: none listed, found, not found.

use crate::errors::Refused;
use crate::framework::{CARRY_FLAG, RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva};
use crate::net::lobby::{
    CONNECTIONS, FLAGS, Part, SAME_GUID, SAVED_CONNECTION, Saved, a, answer, dispatch_lobby, dword,
    g, p, run_both, test_eax,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "net::find_saved_connection",
    rva(0x4185DF),
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

const CONNECTION_ENTRY: u32 = 0x418530; // net::connection_entry

/// The Rust version: the registers it leaves.
pub fn find_saved_connection(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if g(CONNECTIONS) != 0 {
        r.ebx = a(SAVED_CONNECTION);
        r.ecx = 0;
        loop {
            r.eax = r.ecx;
            part(CONNECTION_ENTRY, &mut r)?;
            r.eax = r.eax.wrapping_add(0x40);
            part(SAME_GUID, &mut r)?;
            if r.eflags & ZERO_FLAG != 0 {
                r.eax = r.ecx;
                test_eax(&mut r);
                return Ok(Regs {
                    ebx: entry.ebx,
                    ecx: entry.ecx,
                    ..r
                });
            }
            r.ecx = r.ecx.wrapping_add(1);
            if r.ecx == g(CONNECTIONS) {
                break;
            }
        }
    }
    // The flags of the compare that came out equal (the count with 0, or the number with
    // the count), and the carry set.
    r.eax = u32::MAX;
    r.set_zero(true);
    crate::net::lobby::set_sign(&mut r, 0);
    r.set_carry(true);
    Ok(Regs {
        ebx: entry.ebx,
        ecx: entry.ecx,
        ..r
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, find_saved_connection);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // push ebx; push ecx; cmp dword_52AC34, 0
    let copy = match crate::net::lobby::copy(
        0x4185DF,
        0x3E,
        &[0x53, 0x51, 0x83, 0x3D, 0x34, 0xAC, 0x52, 0x00, 0x00],
        &[0x15, 0x1D],
        &[CONNECTION_ENTRY, SAME_GUID],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [dword(CONNECTIONS)];
    let saved = Saved::new(&all);
    for (count, same) in [(0u32, 9u32), (3, 1), (3, 0), (3, 9), (1, 9)] {
        run_both(
            &mut t,
            &format!("{count} listed, the same at {same}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            FLAGS,
            &[],
            &|| {
                p(CONNECTIONS, count);
                answer(Box::new(move |place, r| {
                    if place == CONNECTION_ENTRY {
                        r.eax = 0x0060_0000 + 0x54 * r.eax;
                    } else {
                        r.set_zero(r.eax == 0x0060_0040 + 0x54 * same);
                        r.set_carry(r.eax & 1 != 0);
                    }
                }));
            },
            &find_saved_connection,
        );
    }
    drop(saved);
    let _ = CARRY_FLAG;
    t.finish()
}
