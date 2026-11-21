//! The controller page's binding items (`menu::learn_binding_1` .. `learn_binding_11`): the
//! item clicked starts its control's learning (`screens::learn_axis` takes it each frame):
//! `controls.learning` 1 (learning), the learning start time the clock (`clock::tick_count`,
//! the 3 s start), the control being learnt the control (1 .. 11). The eleventh is in no menu,
//! and nothing refers to it.
//!
//! The learning takes the controller's reading at its start as its rest
//! (`input::profiles::begin_learning`): only what moves from there counts, so an axis that
//! rests at an end of its travel (a pedal) is not a control pressed.
//!
//! The controller pages' items run `learn_binding(n)` (`menu::table_routes`).

use crate::screens::controls_store::ControlsScreen;

/// Control `control` learns from now.
pub(super) fn learn_binding(
    controls: &ControlsScreen,
    picture_clock: &crate::clock::tick_count::PictureClock,
    input: &crate::input::owner::Input,
    control: u32,
) {
    crate::input::profiles::begin_learning(input);
    let l = rc2k_core::input::devices::Learning::start(
        control,
        crate::clock::tick_count::tick_count(picture_clock),
    );
    controls.learning.set(l.on);
    controls.learn_start.set(l.started);
    controls.learned.set(l.control);
}
