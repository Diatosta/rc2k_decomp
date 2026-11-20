//! Unless the block's view is held (`camera_switch_active`), the next view:
//! `camera_view` + 1, but views 2 and 3 become 4 when the car has none (`kind_has_views`[car]` 0, or the car
//! number `car_kind` past 0x21) or with two players (`players` 2); 7 and above wrap to 0; into
//! `camera_blend_view`, then `drive::set_camera_view`. It is the change-view key's action.

use crate::drive::set_camera_view::set_view_at;
use crate::errors::BadInput;
use crate::memory::session::PlainDword;
use crate::physics::car_store::Car;
use crate::race::kind_store::KindTables;
use crate::race::owner::Race;
use rc2k_core::drive::camera_views::next_view;

pub(super) fn next_view_at(
    kinds: &KindTables,
    car: &Car,
    players: &PlainDword,
    step: &PlainDword,
) -> Result<(), BadInput> {
    if car.camera_switch_active.get() != 0 {
        return Ok(());
    }
    let view = next_view(
        car.camera_view.get(),
        || car.car_kind.get(),
        |car| kinds.kind_has_views[car as usize].get() != 0, /* by car (0x22) */
        || players.get(),
    );
    car.camera_blend_view.set(view);
    set_view_at(car, step)
}

pub fn next_camera_view(race: &Race, car: &Car) -> Result<(), BadInput> {
    let shared = &race.shared;
    next_view_at(
        race.kind_tables(),
        car,
        &shared.view.get().players,
        &shared.car().step_clock,
    )
}
