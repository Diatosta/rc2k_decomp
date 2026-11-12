//! The models' light records, 0x5C906C .. 0x5CB954: `g_lightRecords`, 34 records of 0x134 bytes
//! (IDA struct `ModelRecord`), one per model (car) kind: its axle figures, the groups'
//! numbers and the lights' data (`race::copy_lights_data` lays the last in from `ldata`).
//! Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::Dword;

/// One model's record (IDA struct `ModelRecord`, 0x134 bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
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
#[derive(rc2k_derive::Cells)]
pub struct LightRecords {
    pub records: [LightRecord; 34],
}

impl LightRecords {
    /// The group number at byte `offset` (of the groups, a multiple of 4) of the record of
    /// model `index`. The original does not bound the index: the place after the 34 records
    /// (the one a kind none of the known ones gets) reads on into the gear figures, where the
    /// record's groups fall on the test drive's speed samples; anything further is refused.
    pub fn group_at(
        &self,
        gear: &crate::physics::gear_store::GearFigures,
        index: u32,
        offset: u32,
    ) -> Result<u32, crate::errors::BadInput> {
        const PAST: &str = "a model's record past the light records";
        let group = (offset / 4) as usize;
        match index.checked_sub(self.records.len() as u32) {
            None => self.records[index as usize]
                .groups
                .get(group)
                .map(crate::session::Dword::get)
                .ok_or(crate::errors::BadInput(PAST)),
            Some(0) => {
                // The groups start 0x58 into the record; the one dword between the records
                // and the gear figures is skipped, and the figures' samples start at +0x38.
                let sample = (0x58 + offset - 4)
                    .checked_sub(0x38)
                    .ok_or(crate::errors::BadInput(PAST))?;
                gear.test_speeds
                    .get((sample / 4) as usize)
                    .map(crate::session::Float::bits)
                    .ok_or(crate::errors::BadInput(PAST))
            }
            Some(_) => Err(crate::errors::BadInput(PAST)),
        }
    }
}

/// The piece: a field of `Frame::lights_and_gear` since 2026-10-29 (it left the race
/// block with its group, phase 13).
#[cfg(test)]
pub struct InGroup;

#[cfg(test)]
impl core::ops::Deref for InGroup {
    type Target = LightRecords;
    fn deref(&self) -> &LightRecords {
        &race().frame.lights_and_gear.light_records
    }
}

#[cfg(test)]
pub static LIGHT_RECORDS: InGroup = InGroup;

/// IDA 0x5C906C, the first byte of the piece.
pub const START: usize = 0x5C906C;
/// IDA 0x5CB954, the end of the piece.
pub const END: usize = 0x5CB954;

const _: () = assert!(END - START == core::mem::size_of::<LightRecords>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    core::ptr::from_ref(&race().frame.lights_and_gear.light_records).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(LightRecord, axles), 0x2c);
        assert_eq!(offset_of!(LightRecord, groups), 0x58);
        assert_eq!(offset_of!(LightRecord, lights), 0xA4);
        assert_eq!(
            core::ptr::from_ref(&LIGHT_RECORDS.records[1]).addr(),
            piece_addr() + 0x134
        );
    }

    #[test]
    fn a_record_past_the_last_reads_on_into_the_gear_figures_and_no_further() {
        // The longest read: a group number of the kind past the 34 (load_object).
        let gear = &race().frame.lights_and_gear.gear;
        assert!(LIGHT_RECORDS.group_at(gear, 34, 0x3C).is_ok());
        assert_eq!(
            LIGHT_RECORDS.group_at(gear, 34, 0).ok(),
            Some(
                race().frame.lights_and_gear.gear.test_speeds[7]
                    .get()
                    .to_bits()
            )
        );
        assert!(LIGHT_RECORDS.group_at(gear, 35, 0).is_err());
        assert!(LIGHT_RECORDS.group_at(gear, 1, 0x1000).is_err());
    }
}
