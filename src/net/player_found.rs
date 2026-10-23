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
//!
//! Hook removed 2026-10-23 (conversion phase 4): DirectPlay is given `callback`, the Rust
//! version; in the hooked DLL the game's own code passes the original. What the header
//! says of the thunk, shadow mode and the self-test was the hook's.

use crate::errors::Refused;
use crate::field;
use crate::framework::Regs;
use crate::net::lobby::{COPY_PRINTABLE, PLAYERS, Part, block, g, p};
use crate::net::types::NetPlayerBlock;

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

/// The callback DirectPlay is given (no hook; 2026-10-23): the Rust version.
pub unsafe extern "system" fn callback(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    crate::net::lobby::callback_entry(
        "net::player_found",
        0x41929E,
        &[a, b, c, d, e],
        player_found,
    )
}
