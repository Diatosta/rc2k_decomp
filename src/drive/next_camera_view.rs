//! `drive::next_camera_view` (0x43AA38): unless the block's view is held (+0x694), the next view:
//! +0x684 + 1, but views 2 and 3 become 4 when the car has none (`dword_5C5AC0[car]` 0, or the car
//! number +0x24 past 0x21) or with two players (`dword_63DB94` 2); 7 and above wrap to 0; into
//! +0x688, then `drive::set_camera_view`. It is the change-view key's action.
//!
//! Takes: the block
//!
//! Callers: `drive::read_car_controls`, `race::race_keys`.

use crate::drive::set_camera_view::set_view_at;
use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::race::kind_store::KIND_TABLES;
use crate::race::owner::Race;
use crate::session::Dword;
use rc2k_core::drive::camera_views::next_view;

/// On block `car`, the players `players`, the step count `step`.
// Converted: `rc2k_core::drive::camera_views::next_view`.
pub fn next_view_at(car: &Car, players: &Dword, step: &Dword) -> Result<(), BadInput> {
    if car.camera_switch_active.get() != 0 {
        return Ok(());
    }
    let view = next_view(
        car.camera_view.get(),
        || car.car_kind.get(),
        |car| KIND_TABLES.kind_has_views[car as usize].get() != 0, /* dword_5C5AC0, by car (0x22) */
        || players.get(),
    );
    car.camera_blend_view.set(view);
    set_view_at(car, step)
}

/// On the block at `s`.
pub fn next_camera_view(race: &Race, s: u32) -> Result<(), BadInput> {
    let shared = &race.shared;
    let car = crate::race::blocks_store::block_at(s, "a car's block")?;
    next_view_at(car, &shared.view.players, &shared.car.step_clock)
}
