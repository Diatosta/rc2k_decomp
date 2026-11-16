//! `menu::switch_stage_group`: the stage choice's group switched (0 <-> 1), then the map drawn
//! again.
//!
//! See `menu/stage_choice.rs`. In three item lists (the stage choice screens). The menus and draw
//! lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::stage_choice::{draw_stage_map, switch_stage_group};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    switch_stage_group(o.menus, &mut || draw_stage_map(o))
}
