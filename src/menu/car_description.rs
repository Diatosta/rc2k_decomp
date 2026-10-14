//! A car's description, 0x78 bytes (`CarDescription` in IDA): 34 of them from 0x756568,
//! the last the Clio of the update disc (car 0x21), pointed at by the table at
//! 0x7564A0 and by the carousels' records (`dword_75A65C`). The figures are in imperial
//! units, in tenths; the strings are string ids. Read by the car specification screens
//! (`screens::draw_car_figures`, `car_caption`, `menu::load_car_background`,
//! `sound::play_car_stream`) and the carousels.

use crate::errors::{BadInput, Refused};
use crate::framework::game_read;
use crate::scene::carousel_pictures::ENTRIES_RVA;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CarDescription {
    /// The car's number: its pictures, background, slides and stream go by it.
    pub car: u32,
    /// String: the car's name.
    pub name: u32,
    pub field_8: u32,
    /// Cubic centimetres.
    pub capacity: u32,
    /// String.
    pub engine_type: u32,
    pub power: u32,
    pub power_rpm: u32,
    pub torque: u32,
    pub torque_rpm: u32,
    pub bore: u32,
    pub stroke: u32,
    /// String.
    pub compression: u32,
    /// Strings.
    pub transmission: u32,
    pub drive: u32,
    pub steering: u32,
    pub front_brake_size: u32,
    pub rear_brake_size: u32,
    /// Strings.
    pub front_brakes: u32,
    pub rear_brakes: u32,
    pub front_suspension: u32,
    pub rear_suspension: u32,
    pub length: u32,
    pub width: u32,
    pub height: u32,
    pub wheelbase: u32,
    pub front_track: u32,
    pub rear_track: u32,
    pub weight: u32,
    /// String: the first of the seven lines of its description.
    pub description: u32,
    pub field_74: u32,
}

const _: () = assert!(std::mem::size_of::<CarDescription>() == 0x78);

/// The carousels' entries: a table of 0x100 description pointers.
const ENTRIES: u32 = 0x100;
const SHOWN_RVA: usize = crate::framework::rva(0x755990); // dword_755990

/// The description of the entry shown (`dword_755990` through `dword_75A65C`).
pub fn shown_description() -> Result<*mut CarDescription, Refused> {
    unsafe {
        let table: u32 = game_read(ENTRIES_RVA);
        let entry: u32 = game_read(SHOWN_RVA);
        if table == 0 || entry >= ENTRIES {
            return Err(BadInput("no carousel records, or an entry past them").into());
        }
        let record = ((table + entry * 4) as *const u32).read_unaligned();
        if record == 0 {
            return Err(BadInput("an entry without its record").into());
        }
        Ok(record as *mut CarDescription)
    }
}

/// A description read where it lies.
///
/// # Safety
/// `description` must point at a description in the game's memory.
pub unsafe fn read(description: *const CarDescription) -> CarDescription {
    unsafe { description.read_unaligned() }
}
