//! What the players' details pages' settings share (0x467AFD .. 0x467BD6, 0x470F05,
//! 0x470F2D): each changes one setting of the current player (`dword_75598C`, the
//! `Player`), wrapping or held, and one self-test for all of them.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, SelfTest, game_write};
use crate::menu::player::{PLAYERS, Player, player_at};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, restore};
use crate::service::state::PLAYER_INDEX_RVA;

/// The current player, or a refusal.
pub fn current() -> Result<*mut Player, Refused> {
    let index: u32 = unsafe { crate::framework::game_read(PLAYER_INDEX_RVA) };
    match player_at(index) {
        0 => Err(BadInput("no current player").into()),
        player => Ok(player as *mut Player),
    }
}

/// One step of a setting: `field` picks it, `step` makes the next value.
pub fn change(field: fn(&mut Player) -> &mut u32, step: fn(u32) -> u32) -> Result<(), Refused> {
    let at = current()?;
    unsafe {
        let mut player = at.read_unaligned();
        let value = field(&mut player);
        *value = step(*value);
        at.write_unaligned(player);
    }
    Ok(())
}

pub use rc2k_core::menu::player::{wrap_down, wrap_up};

/// A setting's self-test: each player, the setting at the given values; `left` is what the
/// registers hold afterwards beyond what is preserved.
pub fn check_setting(
    t: &mut SelfTest,
    call_original: unsafe extern "C" fn(*mut Regs),
    field: fn(&mut Player) -> &mut u32,
    values: &[u32],
    rust: &dyn Fn(&Regs) -> Result<Left, Refused>,
) {
    let players: Vec<Region> = (0..PLAYERS)
        .map(player_at)
        .filter(|&player| player != 0)
        .map(|player| (player, std::mem::size_of::<Player>()))
        .collect();
    let inputs = [at(PLAYER_INDEX_RVA)];
    let saved = (capture(&players), capture(&inputs));
    for index in 0..PLAYERS {
        let player = player_at(index);
        if player == 0 {
            continue;
        }
        let fresh = unsafe { (player as *const Player).read_unaligned() };
        for &value in values {
            check_state(
                t,
                &format!("player {index}, value {value:#x}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    game_write(PLAYER_INDEX_RVA, index);
                    let mut fields = fresh;
                    *field(&mut fields) = value;
                    (player as *mut Player).write_unaligned(fields);
                },
                &[(player, std::mem::size_of::<Player>())],
                rust,
            );
        }
    }
    restore(&players, &saved.0);
    restore(&inputs, &saved.1);
}
