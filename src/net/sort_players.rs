//! Copies and sorts the eight players' blocks for the lobby's table by total, then race time,
//! then DirectPlay id (smaller first; a player with no time goes after one with a time). More
//! than eight players are refused.

use crate::cells::Store;
use crate::errors::{BadInput, Refused};
use crate::net::store::NetShared;
use crate::net::store::PlayerBlock;
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
    // Two player blocks swapped as dwords: whole blocks of bytes.
    let mut first = vec![0u8; BLOCK as usize];
    let mut second = vec![0u8; BLOCK as usize];
    Store::load(one, 0, &mut first)?;
    Store::load(two, 0, &mut second)?;
    Store::store(one, 0, &second)?;
    Store::store(two, 0, &first)?;
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

/// The players' blocks sorted for the lobby's table (the first `players` of them).
pub fn sort_players(vars: &NetShared) -> Result<[PlayerBlock; 8], Refused> {
    let players = vars.players.get();
    if players > players::MOST {
        return Err(BadInput("more than eight players").into());
    }
    let mut all = vec![0u8; ALL as usize];
    Store::load(&vars.received, 0, &mut all)?;
    let sorted = [const { PlayerBlock::zero() }; 8];
    Store::store(&sorted, 0, &all)?;
    // The logic is in `rc2k_core::net::players` (`bubble`, `goes_after`).
    // `k + 1` is at most 7: `players` is at most eight.
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
    Ok(sorted)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sorted copy holds the blocks in the table's order and the received blocks stay as
    /// they were.
    #[test]
    fn sorts_a_copy_and_leaves_the_received_blocks() {
        let vars = Box::new(NetShared::new());
        vars.players.set(3);
        for (n, (total, id)) in [(30u32, 7u32), (10, 9), (20, 8)].into_iter().enumerate() {
            vars.received[n].record.total.set(total);
            vars.received[n].record.race_time.set(5);
            vars.received[n].id.set(id);
        }
        let sorted = sort_players(&vars).unwrap();
        let ids: Vec<u32> = sorted.iter().take(3).map(|b| b.id.get()).collect();
        assert_eq!(ids, [9, 8, 7]);
        let received: Vec<u32> = vars.received.iter().take(3).map(|b| b.id.get()).collect();
        assert_eq!(received, [7, 9, 8]);
    }
}
