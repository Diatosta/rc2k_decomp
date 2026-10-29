//! Race-block pieces that left the block together (2026-10-29, phase 13): the census of
//! accesses by address saw the original reach across them, so each group keeps its
//! pieces and the bytes between them in the image's layout, one static with the image's
//! first values. Each piece's store is a field here (`race::store::LEFT` lists the groups).
//!
//! - `CARS_AND_REPLAY` (IDA 0x70F3EC .. 0x719FD8): the cars' blocks and the replay's work.
//!   The replay's state is 0xA04 dwords from car block 8 (0x7172EC), running into the
//!   replay's work (`drive::flush_replay`, `drive::load_replay_frame`).
//! - `VIEWER_AND_KINDS` (IDA 0x5C291C .. 0x5C5CE0): the viewer's panel, the block's bytes
//!   after it, the kinds' tables. The effects read the marks' settings as one record from
//!   `asc_5C2F84` (in the panel) to +0x26A8 (in the kinds' tables: `effects::init_marks`).
//! - `LIGHTS_AND_GEAR` (IDA 0x5C906C .. 0x5CC198): the light records, a dword, the gear
//!   figures. `scene::load_object` reads a kind's record past the 34 (an unbounded kind,
//!   as the original) into the gear figures.

use crate::physics::gear_store::GearFigures;
use crate::race::blocks_store::CarBlocks;
use crate::race::kind_store::KindTables;
use crate::race::light_records_store::LightRecords;
use crate::race::replay_work_store::ReplayWork;
use crate::race::viewer_screen_store::ViewerPanelBlock;
use crate::session::Dword;
use core::mem::{offset_of, size_of};

/// IDA 0x70F3EC .. 0x719FD8.
#[repr(C)]
pub struct CarsAndReplay {
    /// +0x0 (0x70F3EC): `race::blocks_store`.
    pub car_blocks: CarBlocks,
    /// +0xA0D0 (0x7194BC): `race::replay_work_store`.
    pub replay_work: ReplayWork,
}

/// IDA 0x5C291C .. 0x5C5CE0.
#[repr(C)]
pub struct ViewerAndKinds {
    /// +0x0 (0x5C291C): `race::viewer_screen_store`.
    pub viewer_panel: ViewerPanelBlock,
    /// +0x6C8 (0x5C2FE4): the block's bytes between the two, by IDA address.
    pub between: [Dword; (0x5C5624 - 0x5C2FE4) / 4],
    /// +0x2D08 (0x5C5624): `race::kind_store`.
    pub kind_tables: KindTables,
}

/// IDA 0x5C906C .. 0x5CC198.
#[repr(C)]
pub struct LightsAndGear {
    /// +0x0 (0x5C906C): `race::light_records_store`.
    pub light_records: LightRecords,
    /// +0x28E8 (0x5CB954): a dword between them, by IDA address.
    pub between: Dword,
    /// +0x28EC (0x5CB958): `physics::gear_store`.
    pub gear: GearFigures,
}

/// The groups' IDA ranges, in order.
pub const CARS_AND_REPLAY_RANGE: (usize, usize) = (0x70F3EC, 0x719FD8);
pub const VIEWER_AND_KINDS_RANGE: (usize, usize) = (0x5C291C, 0x5C5CE0);
pub const LIGHTS_AND_GEAR_RANGE: (usize, usize) = (0x5C906C, 0x5CC198);

const _: () = {
    use crate::{physics, race};
    assert!(size_of::<CarsAndReplay>() == CARS_AND_REPLAY_RANGE.1 - CARS_AND_REPLAY_RANGE.0);
    assert!(
        offset_of!(CarsAndReplay, replay_work)
            == race::replay_work_store::START - race::blocks_store::START
    );
    assert!(race::blocks_store::START == CARS_AND_REPLAY_RANGE.0);
    assert!(race::replay_work_store::END == CARS_AND_REPLAY_RANGE.1);
    assert!(size_of::<ViewerAndKinds>() == VIEWER_AND_KINDS_RANGE.1 - VIEWER_AND_KINDS_RANGE.0);
    assert!(
        offset_of!(ViewerAndKinds, kind_tables)
            == race::kind_store::START - race::viewer_screen_store::START
    );
    assert!(race::viewer_screen_store::START == VIEWER_AND_KINDS_RANGE.0);
    assert!(race::kind_store::END == VIEWER_AND_KINDS_RANGE.1);
    assert!(size_of::<LightsAndGear>() == LIGHTS_AND_GEAR_RANGE.1 - LIGHTS_AND_GEAR_RANGE.0);
    assert!(
        offset_of!(LightsAndGear, gear)
            == physics::gear_store::START - race::light_records_store::START
    );
    assert!(race::light_records_store::START == LIGHTS_AND_GEAR_RANGE.0);
    assert!(physics::gear_store::END == LIGHTS_AND_GEAR_RANGE.1);
};

// SAFETY (the three): every field is a cell over a plain integer (`session`'s cells and
// records of them): any bit pattern is a valid value.
unsafe impl crate::pointers::Cells for CarsAndReplay {}
unsafe impl crate::pointers::Cells for ViewerAndKinds {}
unsafe impl crate::pointers::Cells for LightsAndGear {}

/// The cars' blocks and the replay's work, with the image's first values.
pub static CARS_AND_REPLAY: CarsAndReplay =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<
        { CARS_AND_REPLAY_RANGE.1 - CARS_AND_REPLAY_RANGE.0 },
    >(CARS_AND_REPLAY_RANGE.0));
/// The viewer's panel .. the kinds' tables, with the image's first values.
pub static VIEWER_AND_KINDS: ViewerAndKinds =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<
        { VIEWER_AND_KINDS_RANGE.1 - VIEWER_AND_KINDS_RANGE.0 },
    >(VIEWER_AND_KINDS_RANGE.0));
/// The light records and the gear figures, with the image's first values.
pub static LIGHTS_AND_GEAR: LightsAndGear =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<
        { LIGHTS_AND_GEAR_RANGE.1 - LIGHTS_AND_GEAR_RANGE.0 },
    >(LIGHTS_AND_GEAR_RANGE.0));

/// Where each group is (`moved::MOVED`'s homes).
pub fn cars_and_replay_home() -> usize {
    crate::pointers::game_address(core::ptr::from_ref(&CARS_AND_REPLAY)) as usize
}
pub fn viewer_and_kinds_home() -> usize {
    crate::pointers::game_address(core::ptr::from_ref(&VIEWER_AND_KINDS)) as usize
}
pub fn lights_and_gear_home() -> usize {
    crate::pointers::game_address(core::ptr::from_ref(&LIGHTS_AND_GEAR)) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        let (a, b) = CARS_AND_REPLAY_RANGE;
        crate::gfx::first_values_are_the_games(&CARS_AND_REPLAY, a, b);
        let (a, b) = VIEWER_AND_KINDS_RANGE;
        crate::gfx::first_values_are_the_games(&VIEWER_AND_KINDS, a, b);
        let (a, b) = LIGHTS_AND_GEAR_RANGE;
        crate::gfx::first_values_are_the_games(&LIGHTS_AND_GEAR, a, b);
    }

    #[test]
    fn groups_have_their_homes() {
        for (start, home) in [
            (CARS_AND_REPLAY_RANGE.0, cars_and_replay_home()),
            (VIEWER_AND_KINDS_RANGE.0, viewer_and_kinds_home()),
            (LIGHTS_AND_GEAR_RANGE.0, lights_and_gear_home()),
        ] {
            assert_eq!(home % 4, 0);
            assert_eq!(crate::moved::place(start), home);
        }
        assert_eq!(
            crate::moved::place(0x5C562C),
            viewer_and_kinds_home() + (0x5C562C - VIEWER_AND_KINDS_RANGE.0)
        );
    }
}
