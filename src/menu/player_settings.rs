//! What the players' details pages' settings share (0x467AFD .. 0x467BD6, 0x470F05,
//! 0x470F2D): each changes one setting of the current player (`dword_75598C`, the
//! `Player`), wrapping or held, and one self-test for all of them.

use crate::errors::{BadInput, Refused};
use crate::menu::player::{Player, player_at};
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
