//! The players' own settings kept aside while a championship runs (an improvement,
//! not in the original). A championship, started or loaded, works on the four
//! players' records and names: its car choice, gearing, controls and set-ups, and a loaded
//! one puts the saved game's records over them whole. The original never put the players'
//! own back, so a championship's choices stayed for every other mode afterwards (and
//! `var\ral.dat` kept them on leaving). Here the records and names are set aside before
//! the championship touches them and put back when it is over or left, on every way out.
//! Saving the championship still saves its own (`saves::gather_savegame` runs inside it).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::player::{PLAYERS, Player};
use crate::menu::player_names::{NAME_LEN, cars, name_bytes, set_name_bytes};

/// The four players' records and names, as they were before a championship.
struct Kept {
    players: [Player; PLAYERS as usize],
    names: [[u8; NAME_LEN]; PLAYERS as usize],
}

fn keep(menus: &Menus) -> Kept {
    let records = cars(&menus.play);
    Kept {
        players: std::array::from_fn(|n| records[n].get()),
        names: std::array::from_fn(|n| name_bytes(menus, n)),
    }
}

fn put_back(menus: &Menus, kept: &Kept) {
    for (record, player) in cars(&menus.play).iter().zip(kept.players) {
        record.set(player);
    }
    for (n, name) in kept.names.iter().enumerate() {
        set_name_bytes(menus, n, name);
    }
}

/// `championship` run with the players' own settings put back afterwards, whether it
/// ends, is left or refuses.
pub fn around_championship(
    menus: &Menus,
    championship: impl FnOnce() -> Result<(), Refused>,
) -> Result<(), Refused> {
    let kept = keep(menus);
    let result = championship();
    put_back(menus, &kept);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::BadInput;

    /// What a championship changes in the records and names is gone after it, whether it
    /// ends or refuses.
    #[test]
    fn a_championship_leaves_the_players_own_settings() {
        let _g = crate::test_support::test_lock();
        let menus = crate::menu::owner::menus();
        let before = keep(menus);
        let change = || {
            let player = &cars(&menus.play)[0];
            player.update(|p| p.gearing = (p.gearing + 1) % 3);
            set_name_bytes(menus, 1, &[b'X'; NAME_LEN]);
        };
        for refuses in [false, true] {
            let result = around_championship(menus, || {
                change();
                if refuses {
                    return Err(BadInput("a refusal inside").into());
                }
                Ok(())
            });
            assert_eq!(result.is_err(), refuses);
            let after = keep(menus);
            assert_eq!(after.players[0].gearing, before.players[0].gearing);
            assert_eq!(after.names, before.names);
        }
    }
}
