//! `net::keep_connection` (0x417661): connection eax kept as the one to use next time: its
//! service provider's GUID (record +0x40, 16 bytes) to `unk_52AC94`, its name (64 bytes)
//! to `unk_52ACA4`.
//!
//! In:  eax = the connection's number
//! Out: every register kept (`pusha` .. `popa`)
//! Callers: `net::start_link`, `net::settings_link`, `net::settings_done`. Its call through
//! `part` (`net::connection_entry`, which may end in its fatal errors). Shadow mode runs the
//! original alone. Self-test: a copy whose call goes to a stand-in (a test record), against
//! the Rust version.

use crate::errors::Refused;
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    Part, SAVED_CONNECTION, SAVED_NAME, Saved, a, answer, dispatch_lobby, fill, movsb, region,
    run_both,
};
use crate::net::types::NetConnection;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::keep_connection", rva(0x417661), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CONNECTION_ENTRY: u32 = 0x418530; // net::connection_entry

/// The Rust version: the registers it leaves (the entry's).
pub fn keep_connection(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(CONNECTION_ENTRY, &mut r)?;
    movsb(
        a(SAVED_CONNECTION),
        r.eax + field!(NetConnection, service_provider),
        0x10,
    );
    movsb(a(SAVED_NAME), r.eax, 0x40);
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, keep_connection);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call sub_418530
    let copy = match crate::net::lobby::copy(
        0x417661,
        0x27,
        &[0x60, 0xE8, 0xC9, 0x0E, 0x00, 0x00],
        &[0x1],
        &[CONNECTION_ENTRY],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let record: Vec<u8> = (0..0x54u8).map(|n| n.wrapping_mul(3)).collect();
    let at = record.as_ptr() as u32;
    let all = [region(SAVED_CONNECTION, 0x50)];
    let saved = Saved::new(&all);
    run_both(
        &mut t,
        "connection 2",
        copy.addr(),
        Regs {
            eax: 2,
            ..Regs::SENTINEL
        },
        &all,
        0,
        &[],
        &|| {
            fill(SAVED_CONNECTION, 0x50, 71);
            answer(Box::new(move |_, r| r.eax = at));
        },
        &keep_connection,
    );
    drop(saved);
    t.finish()
}
