//! `net::allocate_lists` (0x418576): room for 0x100 connections (0x5400 bytes, to
//! `dword_52AC38`) and 0x100 sessions (0x6800 bytes, to `dword_52ACF8`) on the network's
//! memory list (`g_netMemory`, `memory::allocate`), and both counts 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! `memory::allocate` coming back with carry: fatal error 0x80E68021 for the first, 0x80E68022
//! for the second. Caller: `net::find_connections`. Its calls through `part`
//! (`net::lobby_calls`: the Rust version). Shadow mode runs the original alone (memory is
//! taken). Self-test: a copy whose calls go to a stand-in and whose jumps to the fatal stubs
//! land in the test, against the Rust version: both taken, either failing.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    ALLOCATE, CONNECTION_TABLE, CONNECTIONS, NET_MEMORY, NO_CONNECTION_MEMORY, NO_SESSION_MEMORY,
    Part, SESSION_TABLE, SESSIONS, Saved, a, answer, dispatch_lobby, dword, fatal, fatal_popa_1,
    fatal_popa_2, fill, p, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::allocate_lists", rva(0x418576), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers it leaves (the entry's).
pub fn allocate_lists(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    for (bytes, table, code) in [
        (0x5400, CONNECTION_TABLE, NO_CONNECTION_MEMORY),
        (0x6800, SESSION_TABLE, NO_SESSION_MEMORY),
    ] {
        r.eax = bytes;
        r.ebx = a(NET_MEMORY);
        part(ALLOCATE, &mut r)?;
        if r.carry() {
            return Err(fatal(code).into());
        }
        p(table, r.eax);
    }
    p(CONNECTIONS, 0);
    p(SESSIONS, 0);
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, allocate_lists);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov eax, 5400h
    let copy = match crate::net::lobby::copy(
        0x418576,
        0x45,
        &[0x60, 0xB8, 0x00, 0x54, 0x00, 0x00],
        &[0xC, 0x27],
        &[ALLOCATE],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    unsafe {
        copy.retarget(0x11 + 2, fatal_popa_1 as *const () as usize);
        copy.retarget(0x2C + 2, fatal_popa_2 as *const () as usize);
    }
    let regions = [
        dword(CONNECTION_TABLE),
        dword(SESSION_TABLE),
        dword(CONNECTIONS),
        dword(SESSIONS),
    ];
    let saved = Saved::new(&regions);
    for fails in [0u32, 1, 2] {
        run_both(
            &mut t,
            &format!("call {fails} failing"),
            copy.addr(),
            Regs::SENTINEL,
            &regions,
            0,
            &[NO_CONNECTION_MEMORY, NO_SESSION_MEMORY],
            &|| {
                fill(CONNECTION_TABLE, 4, 3);
                let mut n = 0;
                answer(Box::new(move |_, r| {
                    n += 1;
                    r.eax = 0x0055_0000 + n;
                    r.set_carry(n == fails);
                }));
            },
            &allocate_lists,
        );
    }
    drop(saved);
    t.finish()
}
