//! Copies and sorts the eight players' blocks for the lobby's table by total, then race time,
//! then DirectPlay id (smaller first; a player with no time goes after one with a time). More
//! than eight players are refused.

use crate::errors::{BadInput, Refused};
use crate::net::owner::Net;
use crate::net::store::{PlayerBlock, PlayerView};
use rc2k_core::net::players::{self, Standing};

/// The block as the lobby's table sorts it.
fn standing(view: &PlayerView) -> Standing {
    Standing {
        total: view.record.total(),
        time: view.record.race_time(),
        id: view.id,
    }
}

/// The two blocks swapped, the copies of their messages with them.
pub(super) fn swap_blocks(one: &PlayerBlock, two: &PlayerBlock) -> Result<(), BadInput> {
    one.swap_with(two);
    Ok(())
}

/// `players::bubble` over the blocks, `swap` refusing kept and raised after the sort.
pub(super) fn bubble_blocks(
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
pub fn sort_players(net: &Net) -> Result<[PlayerView; 8], Refused> {
    let vars = &net.vars;
    let players = net.race_slot.players();
    if players > players::MOST {
        return Err(BadInput("more than eight players").into());
    }
    let sorted = std::cell::RefCell::new(std::array::from_fn(|n| vars.received[n].view()));
    // The logic is in `rc2k_core::net::players` (`bubble`, `goes_after`).
    // `k + 1` is at most 7: `players` is at most eight.
    bubble_blocks(
        players,
        |k| {
            let table = sorted.borrow();
            players::goes_after(
                standing(&table[k as usize]),
                standing(&table[k as usize + 1]),
            )
        },
        |k| {
            sorted.borrow_mut().swap(k as usize, k as usize + 1);
            Ok(())
        },
    )?;
    Ok(sorted.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sorted copy holds the blocks in the table's order and the received blocks stay as
    /// they were.
    #[test]
    fn sorts_a_copy_and_leaves_the_received_blocks() {
        let _lock = crate::test_support::test_lock();
        let net = Box::new(Net::new());
        let vars = &net.vars;
        net.race_slot.set_players(3);
        for (n, (total, id)) in [(30u32, 7u32), (10, 9), (20, 8)].into_iter().enumerate() {
            vars.received[n].record.update(|r| {
                r.set_total(total);
                r.set_race_time(5);
            });
            vars.received[n].id.set(id);
        }
        let sorted = sort_players(&net).unwrap();
        let ids: Vec<u32> = sorted.iter().take(3).map(|b| b.id).collect();
        assert_eq!(ids, [9, 8, 7]);
        let received: Vec<u32> = vars.received.iter().take(3).map(|b| b.id.get()).collect();
        assert_eq!(received, [7, 9, 8]);
    }
}
