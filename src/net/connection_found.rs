//! `net::connection_found` (0x4186E5): DirectPlay's `EnumConnections` callback: "NET: Found"
//! and the connection's name logged; with `g_noticeKind` 2, the two service providers
//! `unk_5AB370` and `unk_5AB380` (`system::same_guid`) left out; else the connection's data
//! copied into memory of the network's list (`memory::allocate`; none: the enumeration
//! stops) and, if fewer than 0x100 are listed, a record added (`net::connection_entry`):
//! +0x50 the data, +0x40 the service provider's GUID, +0 the name
//! (`strings::copy_printable`, 0x40 room).
//!
//! In:  (stdcall) the service provider's GUID, the connection, its size, its name (a
//!      DPNAME: +8 the short name), the flags, the context
//! Out: eax = 1 to go on, 0 to stop; ebx, esi, edi, ebp kept; `ret 18h`
//! A full list stops the enumeration after the memory is taken (it stays on the list).
//! Caller: DirectPlay, from `net::find_connections`. Its thunk is a `stdcall` Rust function
//! (`net::lobby::dispatch_callback`). Its calls through `part`. Shadow mode runs the original
//! alone. Self-test: a copy called with test arguments, its calls to stand-ins (the GUIDs
//! compared, the memory given, the records in a test table, the name copied), against the
//! Rust version: each way out.

use crate::errors::Refused;
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, ZERO_FLAG, rva};
use crate::net::lobby::{
    ALLOCATE, CONNECTIONS, COPY_PRINTABLE, LOG_L2, LOG_L2_NL, NET_MEMORY, NOTICE_KIND, Part,
    SAME_GUID, Saved, a, answer, dispatch_callback, dword, g, gb, log_text, movsb, p, pb, region,
    run_callback,
};
use crate::net::types::NetConnection;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

unsafe extern "system" fn thunk(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> u32 {
    dispatch_callback(
        &HOOK,
        ORIGINAL.load(Ordering::SeqCst),
        &[a, b, c, d, e, f],
        connection_found,
    )
}

type Thunk = unsafe extern "system" fn(u32, u32, u32, u32, u32, u32) -> u32;

pub static HOOK: RawHook = RawHook::new(
    "net::connection_found",
    rva(0x4186E5),
    unsafe { std::mem::transmute::<Thunk, unsafe extern "C" fn()>(thunk) },
    &ORIGINAL,
)
.self_test(self_test);

const FOUND: usize = 0x5C2E75; // "NET: Found"
const LEFT_OUT: [usize; 2] = [0x5AB370, 0x5AB380]; // unk_5AB370, unk_5AB380
const CONNECTION_ENTRY: u32 = 0x418530; // net::connection_entry

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The Rust version: eax.
pub fn connection_found(args: &[u32], part: Part) -> Result<u32, Refused> {
    let (guid, data, size, name) = (args[0], args[1], args[2], args[3]);
    let mut r = Regs::default();
    log_text(LOG_L2, a(FOUND), &r, part)?;
    let short = get(name + 8);
    log_text(LOG_L2_NL, short, &r, part)?;
    if gb(NOTICE_KIND) == 2 {
        for left_out in LEFT_OUT {
            r.eax = guid;
            r.ebx = a(left_out);
            part(SAME_GUID, &mut r)?;
            if r.eflags & ZERO_FLAG != 0 {
                return Ok(1);
            }
        }
    }
    r.eax = size;
    r.ebx = a(NET_MEMORY);
    part(ALLOCATE, &mut r)?;
    if r.carry() {
        return Ok(0);
    }
    let memory = r.eax;
    let count = g(CONNECTIONS);
    if count >= 0x100 {
        return Ok(0);
    }
    p(CONNECTIONS, count + 1);
    let mut e = Regs {
        eax: count,
        ebx: memory,
        ..r
    };
    part(CONNECTION_ENTRY, &mut e)?;
    let record = e.eax;
    unsafe { ((record + field!(NetConnection, data)) as *mut u32).write_unaligned(memory) };
    movsb(memory, data, size);
    movsb(record + field!(NetConnection, service_provider), guid, 0x10);
    let mut c = Regs {
        eax: short,
        ebx: record,
        ecx: 0x40,
        ..e
    };
    part(COPY_PRINTABLE, &mut c)?;
    Ok(1)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [
        LOG_L2,
        LOG_L2_NL,
        SAME_GUID,
        ALLOCATE,
        CONNECTION_ENTRY,
        COPY_PRINTABLE,
    ];
    // push ebp; mov ebp, esp; push ebx; push esi
    let copy = match crate::net::lobby::copy(
        0x4186E5,
        0xAF,
        &[0x55, 0x8B, 0xEC, 0x53, 0x56],
        &[0xD, 0x1A, 0x32, 0x3F, 0x4F, 0x6B, 0x9B],
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut memory = vec![0u8; 0x400];
    let m = memory.as_mut_ptr() as u32;
    // +0 the GUID, +0x10 the data (0x20 bytes), +0x40 the DPNAME, +0x60 the name, +0x80 the
    // memory given, +0x100 the table.
    let all = [dword(CONNECTIONS), region(NOTICE_KIND, 1), (m, 0x400)];
    let saved = Saved::new(&all);
    // (what, notice kind, which provider it is (0, 1 the left out, 2 another), memory
    // given, the count)
    let cases: [(&str, u8, u32, bool, u32); 6] = [
        ("listed", 0, 0, true, 2),
        ("kind 2, another provider", 2, 2, true, 0),
        ("kind 2, the first left out", 2, 0, true, 0),
        ("kind 2, the second left out", 2, 1, true, 0),
        ("no memory", 0, 2, false, 0),
        ("a full list", 0, 2, true, 0x100),
    ];
    for (what, kind, provider, given, count) in cases {
        run_callback(
            &mut t,
            what,
            copy.addr(),
            &[m, m + 0x10, 0x20, m + 0x40, 0, 0],
            &all,
            &|| unsafe {
                for n in 0..0x400u32 {
                    ((m + n) as *mut u8).write((n * 13) as u8);
                }
                if provider < 2 {
                    movsb(m, a(LEFT_OUT[provider as usize]), 0x10);
                }
                ((m + 0x48) as *mut u32).write(m + 0x60);
                std::ptr::copy_nonoverlapping(
                    c"Serial".as_ptr() as *const u8,
                    (m + 0x60) as *mut u8,
                    7,
                );
                pb(NOTICE_KIND, kind);
                p(CONNECTIONS, count);
                answer(Box::new(move |place, r| match place {
                    SAME_GUID => {
                        let same = crate::system::same_guid::same_guid(r.eax, r.ebx);
                        r.set_zero(same);
                    }
                    ALLOCATE => {
                        r.eax = m + 0x80;
                        r.set_carry(!given);
                    }
                    CONNECTION_ENTRY => r.eax = m + 0x100 + 0x54 * (r.eax & 3),
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
            connection_found,
        );
    }
    drop(saved);
    let _ = &mut memory;
    t.finish()
}
