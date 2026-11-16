//! The controller page's binding items (`menu::learn_binding_1` .. `learn_binding_11`): the
//! item clicked starts its control's learning (`screens::learn_axis` takes it each frame):
//! `controls.learning` 1 (learning), the learning start time the clock (`system::tick_count`,
//! the 3 s start), the control being learnt the control (1 .. 11). The eleventh is in no menu,
//! and nothing refers to it.
//!
//! The learning takes the axes' values at its start as their rest (`input::axes::axis_moved`
//! with 0), and the page's direction history is emptied.
//!
//! The controller pages' items run `learn_binding(n)` (`menu::table_routes`).

use crate::screens::controls_store::ControlsScreen;

/// Control `control` learns from now.
pub fn learn_binding(
    controls: &ControlsScreen,
    system: &crate::system::owner::System,
    input: &crate::input::owner::Input,
    control: u32,
) {
    // Only what moves from where the controller rests now counts: an axis resting at an end
    // of its travel (a pedal) is not a direction held.
    let _ = crate::input::axes::axis_moved(0, input);
    for cell in &controls.history {
        cell.set(0xFFFF_FFFF);
    }
    let l = rc2k_core::input::devices::Learning::start(
        control,
        crate::system::tick_count::tick_count(&system.session.picture_clock),
    );
    controls.learning.set(l.on);
    controls.learn_start.set(l.started);
    controls.learned.set(l.control);
}
