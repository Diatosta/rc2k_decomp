//! `net::sort_players` (0x418108): the eight players' blocks of `g_netReceived` copied to
//! `unk_56AFFC` and bubble-sorted there for the lobby's table: by the total (+0x30), then
//! the race's time (+0x2C), then the DirectPlay id (+0x20), unsigned, the smaller first; a
//! player whose time is -1 (no time) goes after one who has a time.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! `g_netPlayers` - 1 passes (signed: none for one player or none). More than eight players
//! would sort past the copy: refused (DirectPlay's listing keeps eight at most). Caller:
//! `net::lobby_frame`.

use crate::errors::{BadInput, Refused};
use crate::net::lobby::{BLOCK, PLAYERS, RECEIVED, SORTED, a, g, movsb};
use rc2k_core::net::players::{self, Standing};

const ALL: u32 = 8 * BLOCK;

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The block at `x` as the lobby's table sorts it.
fn standing(x: u32) -> Standing {
    Standing {
        total: get(x + 0x30),
        time: get(x + 0x2C),
        id: get(x + 0x20),
    }
}

/// The blocks at `x` and the next swapped, a dword at a time.
pub fn swap_blocks(x: u32) {
    for n in 0..BLOCK / 4 {
        let (one, two) = (x + 4 * n, x + BLOCK + 4 * n);
        let v = get(one);
        unsafe {
            (one as *mut u32).write_unaligned(get(two));
            (two as *mut u32).write_unaligned(v);
        }
    }
}

/// The Rust version (the original keeps every register).
pub fn sort_players() -> Result<(), Refused> {
    let players = g(PLAYERS);
    if players > players::MOST {
        return Err(BadInput("more than eight players").into());
    }
    movsb(a(SORTED), a(RECEIVED), ALL);
    // Converted: `rc2k_core::net::players` (`bubble`, `goes_after`).
    let x = |k: u32| a(SORTED) + k * BLOCK;
    players::bubble(
        players,
        |k| players::goes_after(standing(x(k)), standing(x(k) + BLOCK)),
        |k| swap_blocks(x(k)),
    );
    Ok(())
}
