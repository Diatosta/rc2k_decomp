//! The service's owner: the car set-ups' stores in one struct, a field of the menus.

use crate::memory::session::{PlainBytes, PlainDword};
use crate::service::read_setup_slot::{SetUps, template};
use std::cell::Cell;

/// One of the service area's four setup items (gear ratio, tyre, ride height, suspension
/// stiffness).
pub struct SetupItem {
    /// The value being chosen (`service::step_*_up` / `_down`).
    pub chosen: Cell<u32>,
    /// The value installed on the car (put back when the change is toggled off).
    pub installed: Cell<u32>,
    /// The value at the start of the item's edit page (`service::begin_edits`).
    pub start: Cell<u32>,
}

impl SetupItem {
    const fn new(value: u32) -> Self {
        SetupItem {
            chosen: Cell::new(value),
            installed: Cell::new(value),
            start: Cell::new(0),
        }
    }
}

/// The handle of repair time `n` among `Service::repair_times`: its number + 1 (0 means
/// none, -1 ends the list).
const fn repair_time(n: u32) -> PlainDword {
    assert!(n < 16);
    PlainDword::new(n + 1)
}

/// The service's state.
pub struct Service {
    /// The set-ups' buffers (file head, names, a kept value).
    pub set_ups: SetUps,
    /// The set-up file's name, the number's two digits last.
    pub template: PlainBytes<27>,
    /// The four setup items: [0] gear ratio, [1] tyre (0 .. 5, its name is string 0x4B0 + it),
    /// [2] ride height (a percentage), [3] suspension stiffness (a percentage).
    pub setups: [SetupItem; 4],
    /// The repair times; 14 values for `menu::driver_stats`, 16 in two columns of eight for
    /// `screens::draw_repair_times` (items 12 .. 15 are `begin_edits`' times).
    pub repair_times: [PlainDword; 16],
    /// The time of the chosen repairs.
    pub repair_time: PlainDword,
    /// `elapsed` at the last repair step (`service::apply_repairs`).
    pub repair_last_step: PlainDword,
    /// The service boxes' flags, the first set, by item value (-1 ends it).
    pub repair_flags: [PlainDword; 21],
    /// The service boxes' flags, the second set, by item value (-1 ends it).
    pub repair_marks: [PlainDword; 21],
    /// The handle of each repair item's time among `repair_times`, by item (`service::state`).
    pub repair_time_handles: [PlainDword; 24],
}

impl Service {
    pub const fn new() -> Self {
        Service {
            set_ups: SetUps::new(),
            template: template(),
            setups: [
                SetupItem::new(2),
                SetupItem::new(3),
                SetupItem::new(0x32),
                SetupItem::new(0x32),
            ],
            repair_times: [const { PlainDword::new(0) }; 16],
            repair_time: PlainDword::new(0),
            repair_last_step: PlainDword::new(0),
            repair_flags: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0xffffffff),
            ],
            repair_marks: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0xffffffff),
            ],
            repair_time_handles: [
                PlainDword::new(0),
                PlainDword::new(0),
                repair_time(0),
                repair_time(1),
                repair_time(2),
                repair_time(3),
                repair_time(4),
                repair_time(5),
                repair_time(6),
                repair_time(7),
                repair_time(8),
                repair_time(9),
                repair_time(10),
                repair_time(11),
                repair_time(12),
                repair_time(13),
                PlainDword::new(0),
                PlainDword::new(0),
                repair_time(14),
                repair_time(15),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0xffffffff),
            ],
        }
    }
}
