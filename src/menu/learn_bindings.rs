//! The controller page's binding items (`menu::learn_binding_1` 0x46AFC6 .. `learn_binding_11`
//! 0x46B110, 0x21 bytes apart): the item clicked starts its control's learning
//! (`screens::learn_axis` takes it each frame): `dword_7551F4` 1 (learning), `dword_7551F8`
//! the clock (`system::tick_count`, the 3 s start), `dword_7551FC` the control (1 .. 11).
//! The eleventh is in no menu, and nothing refers to it (IDA, 2026-10-24).
//!
//! Callers: the actions of the four controller pages' items (0x749C70 ..).
//! The controller pages' items run `learn_binding(n)` (`menu::table_routes`).

use crate::screens::owner::screens;

/// The Rust version: control `control` learns from now.
pub fn learn_binding(control: u32) {
    let screens = screens();
    // Converted: `rc2k_core::input::devices::Learning`.
    let l = rc2k_core::input::devices::Learning::start(
        control,
        crate::system::tick_count::tick_count(),
    );
    screens.controls.learning.set(l.on);
    screens.controls.learn_start.set(l.started);
    screens.controls.learned.set(l.control);
}
