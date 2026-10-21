//! `net::session_found` (0x418A62): DirectPlay's `EnumSessions` callback: "NET: Found", the
//! flags, the session's name logged; on a time-out (flag 1) or no session the enumeration
//! stops; a session whose `dwUser2` bit 0 is clear is passed over; else, if fewer than
//! 0x100 are listed, a record added (`net::session_entry`): the name
//! (`strings::copy_printable`, 0x40 room), the instance GUID (+0x40), the most players
//! (+0x50), the players (+0x54), `dwUser1` .. `dwUser4` (+0x58).
//!
//! In:  (stdcall) the session's description (a DPSESSIONDESC2), the time-out's address,
//!      the flags, the context
//! Out: eax = 1 to go on, 0 to stop; ebx, esi, edi, ebp kept; `ret 10h`
//! Caller: DirectPlay, from `net::find_sessions`. Its thunk is a `stdcall` Rust function
//! (`net::lobby::dispatch_callback`). Its calls through `part`. Shadow mode runs the
//! original alone. Self-test: a copy called with test arguments, its calls to stand-ins
//! (the records in a test table, the name copied), against the Rust version: each way out.

use crate::errors::Refused;
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, rva};
use crate::net::lobby::{
    COPY_PRINTABLE, LOG_HEX_L2, LOG_L2, LOG_NEWLINE_L2, Part, SESSIONS, Saved, a, answer,
    dispatch_callback, dword, g, log_text, movsb, p, run_callback,
};
use crate::net::types::{DpSessionDesc, NetSession};
use rc2k_core::net::lists::{self, Found};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

unsafe extern "system" fn thunk(a: u32, b: u32, c: u32, d: u32) -> u32 {
    dispatch_callback(
        &HOOK,
        ORIGINAL.load(Ordering::SeqCst),
        &[a, b, c, d],
        session_found,
    )
}

type Thunk = unsafe extern "system" fn(u32, u32, u32, u32) -> u32;

pub static HOOK: RawHook = RawHook::new(
    "net::session_found",
    rva(0x418A62),
    unsafe { std::mem::transmute::<Thunk, unsafe extern "C" fn()>(thunk) },
    &ORIGINAL,
)
.self_test(self_test);

const FOUND: usize = 0x5C2E75; // "NET: Found"
const SESSION_ENTRY: u32 = 0x418553; // net::session_entry

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The Rust version: eax.
pub fn session_found(args: &[u32], part: Part) -> Result<u32, Refused> {
    let (desc, flags) = (args[0], args[2]);
    let r = Regs::default();
    log_text(LOG_L2, a(FOUND), &r, part)?;
    log_text(LOG_HEX_L2, flags, &r, part)?;
    if desc != 0 {
        log_text(LOG_L2, get(desc + 0x30), &r, part)?;
    }
    let mut n = r;
    if desc != 0 {
        n.eax = get(desc + 0x30);
    }
    part(LOG_NEWLINE_L2, &mut n)?;
    // Converted: `rc2k_core::net::lists::session_found`.
    let count = match lists::session_found(flags, desc != 0, || get(desc + 0x44), || g(SESSIONS)) {
        Found::Stop => return Ok(0),
        Found::Pass => return Ok(1),
        Found::List(n) => n,
    };
    p(SESSIONS, count + 1);
    let mut e = Regs { eax: count, ..r };
    part(SESSION_ENTRY, &mut e)?;
    let record = e.eax;
    let mut c = Regs {
        eax: get(desc + 0x30),
        ebx: record,
        ecx: 0x40,
        ..e
    };
    part(COPY_PRINTABLE, &mut c)?;
    movsb(
        record + field!(NetSession, instance),
        desc + field!(DpSessionDesc, instance),
        0x10,
    );
    unsafe {
        ((record + field!(NetSession, max_players)) as *mut u32)
            .write_unaligned(get(desc + field!(DpSessionDesc, max_players)));
        ((record + field!(NetSession, players)) as *mut u32)
            .write_unaligned(get(desc + field!(DpSessionDesc, current_players)));
    }
    movsb(
        record + field!(NetSession, user),
        desc + field!(DpSessionDesc, user),
        0x10,
    );
    Ok(1)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [
        LOG_L2,
        LOG_HEX_L2,
        LOG_NEWLINE_L2,
        SESSION_ENTRY,
        COPY_PRINTABLE,
    ];
    // push ebp; mov ebp, esp; push ebx; push esi
    let copy = match crate::net::lobby::copy(
        0x418A62,
        0xB1,
        &[0x55, 0x8B, 0xEC, 0x53, 0x56],
        &[0xD, 0x17, 0x28, 0x2E, 0x5F, 0x73],
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut memory = vec![0u8; 0x400];
    let m = memory.as_mut_ptr() as u32;
    // +0 the description (0x50), +0x60 the name, +0x100 the table
    let all = [dword(SESSIONS), (m, 0x400)];
    let saved = Saved::new(&all);
    // (what, a description, the flags, dwUser2, the count)
    let cases: [(&str, bool, u32, u32, u32); 6] = [
        ("listed", true, 0, 1, 1),
        ("timed out", true, 1, 1, 0),
        ("no description", false, 0, 1, 0),
        ("passed over", true, 0, 2, 0),
        ("a full list", true, 0, 3, 0x100),
        ("listed, flags 4", true, 4, 0xFFFF_FFFF, 0),
    ];
    for (what, with, flags, user2, count) in cases {
        run_callback(
            &mut t,
            what,
            copy.addr(),
            &[if with { m } else { 0 }, 0, flags, 0],
            &all,
            &|| unsafe {
                for n in 0..0x400u32 {
                    ((m + n) as *mut u8).write((n * 7) as u8);
                }
                ((m + 0x30) as *mut u32).write(m + 0x60);
                ((m + 0x44) as *mut u32).write(user2);
                std::ptr::copy_nonoverlapping(
                    c"Rally".as_ptr() as *const u8,
                    (m + 0x60) as *mut u8,
                    6,
                );
                p(SESSIONS, count);
                answer(Box::new(move |place, r| match place {
                    SESSION_ENTRY => r.eax = m + 0x100 + 0x68 * (r.eax & 3),
                    COPY_PRINTABLE => {
                        let _ = crate::strings::copy_printable::copy_printable(
                            r.eax as *const u8,
                            r.ebx as *mut u8,
                            r.ecx,
                        );
                    }
                    _ => {}
                }));
            },
            session_found,
        );
    }
    drop(saved);
    let _ = &mut memory;
    t.finish()
}
