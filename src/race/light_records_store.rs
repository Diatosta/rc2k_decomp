//! The models' light records, 0x5C906C .. 0x5CB954: `g_lightRecords`, 34 records of 0x134 bytes
//! (IDA struct `ModelRecord`), one per model (car) kind: its axle figures, the groups'
//! numbers and the lights' data (`race::copy_lights_data` lays the last in from `ldata`).
//! Names are readings unless IDA's.

use crate::session::Dword;

/// One model's record (IDA struct `ModelRecord`, 0x134 bytes).
#[repr(C)]
pub struct LightRecord {
    /// +0x0: `field_0`: eleven dwords (read at +0xC, +0x10 .. +0x28 by `physics::update_camera`,
    /// `race::car_draw`, `drive::share_car_state`).
    pub head: [Dword; 11],
    /// +0x2C: `axles`: the five axle values (`race::car_parts`) and the six listener figures
    /// from +0x40 (`drive::set_listener`).
    pub axles: [Dword; 11],
    /// +0x58: `groups`: the group numbers of the model's parts (`scene::load_object`,
    /// `scene::list_object_face`, `effects::init_mark_step`, `race::car_draw`).
    pub groups: [Dword; 19],
    /// +0xA4: `lights`: 0x90 bytes of the lights' data (`race::copy_lights_data`,
    /// `race::frame_draw`).
    pub lights: [Dword; 36],
}

const _: () = assert!(core::mem::size_of::<LightRecord>() == 0x134);

/// The models' records.
#[repr(C)]
pub struct LightRecords {
    pub records: [LightRecord; 34],
}

impl LightRecords {
    /// Where record `index` is, by wrapping arithmetic on the first record's address (the
    /// game's code does not bound the index: callers that read an offset of the record, or
    /// pass it to x87 runs, use this).
    pub fn record_addr(&self, index: u32) -> u32 {
        (self.records[0].head[0].addr()).wrapping_add(index.wrapping_mul(0x134))
    }
}

/// The piece: a field of `race::groups_store::LIGHTS_AND_GEAR` since 2026-10-29 (it left the race
/// block with its group, phase 13).
pub struct InGroup;

impl core::ops::Deref for InGroup {
    type Target = LightRecords;
    fn deref(&self) -> &LightRecords {
        &crate::race::groups_store::LIGHTS_AND_GEAR.light_records
    }
}

pub static LIGHT_RECORDS: InGroup = InGroup;

/// IDA 0x5C906C, the first byte of the piece.
pub const START: usize = 0x5C906C;
/// IDA 0x5CB954, the end of the piece.
pub const END: usize = 0x5CB954;

const _: () = assert!(END - START == core::mem::size_of::<LightRecords>());

/// Where IDA 0x5C906C is now (its group's static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &crate::race::groups_store::LIGHTS_AND_GEAR.light_records,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(LightRecord, axles), 0x5C9098 - START);
        assert_eq!(offset_of!(LightRecord, groups), 0x5C90C4 - START);
        assert_eq!(offset_of!(LightRecord, lights), 0xA4);
        assert_eq!(LIGHT_RECORDS.record_addr(1), (piece_addr() + 0x134));
    }
}
