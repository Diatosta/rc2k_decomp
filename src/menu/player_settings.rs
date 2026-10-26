use crate::errors::{BadInput, Refused};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::player::{Player, player_at};

/// The address of the current player's record, or a refusal.
pub fn current() -> Result<u32, Refused> {
    let index: u32 = EVENT_STATE.current_player.get();
    match player_at(index) {
        0 => Err(BadInput("no current player").into()),
        player => Ok(player),
    }
}

/// One step of a setting: `field` picks it, `step` makes the next value.
pub fn change(field: fn(&mut Player) -> &mut u32, step: fn(u32) -> u32) -> Result<(), Refused> {
    let at = current()?;
    let at = crate::pointers::GamePtr::new(at);
    let mut player = at.read::<Player>("the current player")?;
    let value = field(&mut player);
    *value = step(*value);
    at.write(player, "the current player")?;
    Ok(())
}
