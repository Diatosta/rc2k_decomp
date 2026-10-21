//! `net::player_found` (0x41929E): DirectPlay's `EnumPlayers` callback: while fewer than eight
//! are listed (`g_netPlayers`), the player's block of `g_netReceived` gets its id (+0x20) and
//! its short name (+0, `strings::copy_printable`, 0x1F room), and the enumeration goes on;
//! with eight it stops.
//!
//! In:  (stdcall) the player's id, its type, its name (a DPNAME: +8 the short name), the
//!      flags, the context
//! Out: eax = 1 go on, 0 stop; ebx, esi, edi, ebp kept; `ret 14h`
//! Caller: DirectPlay, from `net::list_players`. Its thunk is a `stdcall` Rust function
//! (`net::lobby::dispatch_callback`). Its call through `part`. Shadow mode runs the original
//! alone. Self-test: a copy called with test arguments, its call to a stand-in (the name
//! copied), against the Rust version: 0, 5 and 8 listed.

use crate::errors::Refused;
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, rva};
use crate::net::lobby::{
    COPY_PRINTABLE, PLAYERS, Part, Saved, answer, block, dispatch_callback, dword, g, p,
    run_callback,
};
use crate::net::types::NetPlayerBlock;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

unsafe extern "system" fn thunk(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    dispatch_callback(
        &HOOK,
        ORIGINAL.load(Ordering::SeqCst),
        &[a, b, c, d, e],
        player_found,
    )
}

type Thunk = unsafe extern "system" fn(u32, u32, u32, u32, u32) -> u32;

pub static HOOK: RawHook = RawHook::new(
    "net::player_found",
    rva(0x41929E),
    unsafe { std::mem::transmute::<Thunk, unsafe extern "C" fn()>(thunk) },
    &ORIGINAL,
)
.self_test(self_test);

/// The Rust version: eax.
pub fn player_found(args: &[u32], part: Part) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::lists::player_found`.
    let Some(count) = rc2k_core::net::lists::player_found(g(PLAYERS)) else {
        return Ok(0);
    };
    p(PLAYERS, count + 1);
    let at = block(count);
    unsafe { ((at + field!(NetPlayerBlock, id)) as *mut u32).write_unaligned(args[0]) };
    let mut r = Regs {
        eax: unsafe { ((args[2] + 8) as *const u32).read_unaligned() },
        ebx: at,
        ecx: 0x1F,
        ..Regs::default()
    };
    part(COPY_PRINTABLE, &mut r)?;
    Ok(1)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // push ebp; mov ebp, esp; push ebx; push esi
    let copy = match crate::net::lobby::copy(
        0x41929E,
        0x4C,
        &[0x55, 0x8B, 0xEC, 0x53, 0x56],
        &[0x38],
        &[COPY_PRINTABLE],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut all = vec![dword(PLAYERS)];
    for n in 0..8 {
        all.push((block(n), field!(NetPlayerBlock, record) as usize));
    }
    let saved = Saved::new(&all);
    let name = b"Colin\0";
    let dp_name = [0x10u32, 0, name.as_ptr() as u32, 0];
    for count in [0u32, 5, 8] {
        run_callback(
            &mut t,
            &format!("{count} listed"),
            copy.addr(),
            &[0x1234, 0, dp_name.as_ptr() as u32, 0, 0],
            &all,
            &|| {
                p(PLAYERS, count);
                answer(Box::new(|_, r| {
                    let _ = unsafe {
                        crate::strings::copy_printable::copy_printable(
                            r.eax as *const u8,
                            r.ebx as *mut u8,
                            r.ecx,
                        )
                    };
                }));
            },
            player_found,
        );
    }
    drop(saved);
    t.finish()
}
