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
use crate::net::lobby::movsb;
use crate::net::store::{NET, PlayerBlock};
use rc2k_core::net::players::{self, Standing};

const BLOCK: u32 = std::mem::size_of::<PlayerBlock>() as u32;
const ALL: u32 = 8 * BLOCK;

/// The block as the lobby's table sorts it.
fn standing(block: &PlayerBlock) -> Standing {
    Standing {
        total: block.record.total.get(),
        time: block.record.race_time.get(),
        id: block.id.get(),
    }
}

/// The two blocks swapped, a dword at a time.
pub fn swap_blocks(one: &PlayerBlock, two: &PlayerBlock) {
    let (one, two) = (
        one as *const PlayerBlock as *mut u32,
        two as *const PlayerBlock as *mut u32,
    );
    for n in 0..BLOCK as usize / 4 {
        unsafe {
            let v = one.add(n).read_volatile();
            one.add(n).write_volatile(two.add(n).read_volatile());
            two.add(n).write_volatile(v);
        }
    }
}

/// The Rust version (the original keeps every register).
pub fn sort_players() -> Result<(), Refused> {
    let players = NET.players.get();
    if players > players::MOST {
        return Err(BadInput("more than eight players").into());
    }
    movsb(
        NET.sorted.as_ptr() as u32,
        NET.received.as_ptr() as u32,
        ALL,
    );
    // Converted: `rc2k_core::net::players` (`bubble`, `goes_after`).
    // `k + 1` is at most 7: `players` is at most eight.
    let sorted = &NET.sorted;
    players::bubble(
        players,
        |k| {
            players::goes_after(
                standing(&sorted[k as usize]),
                standing(&sorted[k as usize + 1]),
            )
        },
        |k| swap_blocks(&sorted[k as usize], &sorted[k as usize + 1]),
    );
    Ok(())
}
