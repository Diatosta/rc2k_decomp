//! The controller page's binding items (`menu::learn_binding_1` 0x46AFC6 .. `learn_binding_11`
//! 0x46B110, 0x21 bytes apart): the item clicked starts its control's learning
//! (`screens::learn_axis` takes it each frame): `dword_7551F4` 1 (learning), `dword_7551F8`
//! the clock (`system::tick_count`, the 3 s start), `dword_7551FC` the control (1 .. 11).
//! The eleventh is in no menu.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the actions of the four controller pages' items (0x749C70 ..).
//! Hooks removed 2026-10-22 (typed tables) but for `learn_binding_11`, which a screen calls by
//! address: the controller pages' items run their `entry` (`menu::table_calls`).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_write, rva};

const LEARNING_RVA: usize = rva(0x7551F4); // dword_7551F4: 1 while a control learns
const STARTED_RVA: usize = rva(0x7551F8); // dword_7551F8: when it began
const CONTROL_RVA: usize = rva(0x7551FC); // dword_7551FC: the control learning

/// The Rust version: control `control` learns from now.
pub fn learn_binding(control: u32) {
    // Converted: `rc2k_core::input::devices::Learning`.
    let l = rc2k_core::input::devices::Learning::start(
        control,
        crate::system::tick_count::tick_count(),
    );
    unsafe {
        game_write(LEARNING_RVA, l.on);
        game_write(STARTED_RVA, l.started);
        game_write(CONTROL_RVA, l.control);
    }
}

/// A binding item without its hook: the bindings the typed tables run (hooks removed
/// 2026-10-22, the eleventh 2026-10-23).
macro_rules! learn_entry {
    ($module:ident, $name:literal, $address:expr, $control:expr) => {
        pub mod $module {
            use super::*;
            /// The Rust version where the typed tables run it (`menu::table_calls`; hook removed
            /// 2026-10-22).
            pub fn run() {
                learn_binding($control);
            }
        }
    };
}

learn_entry!(learn_binding_1, "menu::learn_binding_1", 0x46AFC6, 1);
learn_entry!(learn_binding_2, "menu::learn_binding_2", 0x46AFE7, 2);
learn_entry!(learn_binding_3, "menu::learn_binding_3", 0x46B008, 3);
learn_entry!(learn_binding_4, "menu::learn_binding_4", 0x46B029, 4);
learn_entry!(learn_binding_5, "menu::learn_binding_5", 0x46B04A, 5);
learn_entry!(learn_binding_6, "menu::learn_binding_6", 0x46B06B, 6);
learn_entry!(learn_binding_7, "menu::learn_binding_7", 0x46B08C, 7);
learn_entry!(learn_binding_8, "menu::learn_binding_8", 0x46B0AD, 8);
learn_entry!(learn_binding_9, "menu::learn_binding_9", 0x46B0CE, 9);
learn_entry!(learn_binding_10, "menu::learn_binding_10", 0x46B0EF, 10);
learn_entry!(learn_binding_11, "menu::learn_binding_11", 0x46B110, 11);
