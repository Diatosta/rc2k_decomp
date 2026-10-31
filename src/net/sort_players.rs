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
use crate::pointers::GamePtr;
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
pub fn swap_blocks(one: &PlayerBlock, two: &PlayerBlock) -> Result<(), BadInput> {
    let (one, two) = (
        GamePtr::new(crate::pointers::game_address(std::ptr::from_ref::<
            PlayerBlock,
        >(one))),
        GamePtr::new(crate::pointers::game_address(std::ptr::from_ref::<
            PlayerBlock,
        >(two))),
    );
    for n in 0..BLOCK / 4 {
        let v = one.offset(4 * n).read::<u32>("a player's block")?;
        let w = two.offset(4 * n).read::<u32>("a player's block")?;
        one.offset(4 * n).write(w, "a player's block")?;
        two.offset(4 * n).write(v, "a player's block")?;
    }
    Ok(())
}

/// `players::bubble` over the blocks, `swap` refusing kept and raised after the sort.
pub fn bubble_blocks(
    count: u32,
    after: impl FnMut(u32) -> bool,
    swap: impl Fn(u32) -> Result<(), BadInput>,
) -> Result<(), BadInput> {
    let refused = std::cell::Cell::new(None);
    players::bubble(count, after, |k| {
        if refused.get().is_none()
            && let Err(e) = swap(k)
        {
            refused.set(Some(e));
        }
    });
    refused.take().map_or(Ok(()), Err)
}

/// The Rust version (the original keeps every register).
pub fn sort_players() -> Result<(), Refused> {
    let players = NET.players.get();
    if players > players::MOST {
        return Err(BadInput("more than eight players").into());
    }
    movsb(
        crate::pointers::game_address(NET.sorted.as_ptr()),
        crate::pointers::game_address(NET.received.as_ptr()),
        ALL,
    )?;
    // Converted: `rc2k_core::net::players` (`bubble`, `goes_after`).
    // `k + 1` is at most 7: `players` is at most eight.
    let sorted = &NET.sorted;
    bubble_blocks(
        players,
        |k| {
            players::goes_after(
                standing(&sorted[k as usize]),
                standing(&sorted[k as usize + 1]),
            )
        },
        |k| swap_blocks(&sorted[k as usize], &sorted[k as usize + 1]),
    )?;
    Ok(())
}
