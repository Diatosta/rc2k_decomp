//! `net::group_found` (0x419391): DirectPlay's `EnumGroups` callback: a group (type 0) whose
//! short name is "susanivanovafanclub" (`strings::same_text`) is the one to send to
//! (`g_netSendTo`), and the enumeration stops; any other: it goes on.
//!
//! In:  (stdcall) the group's id, its type, its name (a DPNAME: +8 the short name), the
//!      flags, the context
//! Out: eax = 0 found (stop), 1 go on; ebx, esi, edi, ebp kept; `ret 14h`
//! Caller: DirectPlay, from `net::join_group`. Its thunk is a `stdcall` Rust function
//! (`net::lobby::dispatch_callback`). Its call through `part`. Shadow mode runs the original
//! alone. Self-test: a copy called with test arguments, its call to a stand-in (the texts
//! compared), against the Rust version: the group, another name, another type.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, ZERO_FLAG, rva};
use crate::net::create_group::GROUP_NAME;
use crate::net::lobby::{
    Part, SAME_TEXT, SEND_TO, Saved, a, answer, dispatch_callback, dword, p, run_callback,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

unsafe extern "system" fn thunk(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    dispatch_callback(
        &HOOK,
        ORIGINAL.load(Ordering::SeqCst),
        &[a, b, c, d, e],
        group_found,
    )
}

type Thunk = unsafe extern "system" fn(u32, u32, u32, u32, u32) -> u32;

pub static HOOK: RawHook = RawHook::new(
    "net::group_found",
    rva(0x419391),
    unsafe { std::mem::transmute::<Thunk, unsafe extern "C" fn()>(thunk) },
    &ORIGINAL,
)
.self_test(self_test);

/// The Rust version: eax.
pub fn group_found(args: &[u32], part: Part) -> Result<u32, Refused> {
    if args[1] != 0 {
        return Ok(1);
    }
    let mut r = Regs {
        eax: unsafe { ((args[2] + 8) as *const u32).read_unaligned() },
        ebx: a(GROUP_NAME),
        ..Regs::default()
    };
    part(SAME_TEXT, &mut r)?;
    if r.eflags & ZERO_FLAG == 0 {
        return Ok(1);
    }
    p(SEND_TO, args[0]);
    Ok(0)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // push ebp; mov ebp, esp; push ebx; push esi
    let copy = match crate::net::lobby::copy(
        0x419391,
        0x37,
        &[0x55, 0x8B, 0xEC, 0x53, 0x56],
        &[0x18],
        &[SAME_TEXT],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [dword(SEND_TO)];
    let saved = Saved::new(&all);
    let same = b"SusanIvanovaFanClub\0";
    let other = b"another\0";
    for (what, kind, name) in [
        ("the group", 0u32, &same[..]),
        ("another name", 0, &other[..]),
        ("another type", 1, &same[..]),
    ] {
        let dp_name = [0x10u32, 0, name.as_ptr() as u32, 0];
        run_callback(
            &mut t,
            what,
            copy.addr(),
            &[0x000B_AD1D, kind, dp_name.as_ptr() as u32, 0, 0],
            &all,
            &|| {
                p(SEND_TO, 7);
                answer(Box::new(|_, r| {
                    let order = unsafe {
                        crate::strings::same_text::compare_texts(
                            r.eax as *const u8,
                            r.ebx as *const u8,
                        )
                    };
                    r.set_zero(order == std::cmp::Ordering::Equal);
                }));
            },
            group_found,
        );
    }
    drop(saved);
    t.finish()
}
