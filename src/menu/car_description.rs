//! A car's description, 0x78 bytes (`CarDescription` in IDA): 34 of them from 0x756568,
//! the last the Clio of the update disc (car 0x21), pointed at by the table at
//! 0x7564A0 and by the carousels' records (`dword_75A65C`). The figures are in imperial
//! units, in tenths; the strings are string ids. Read by the car specification screens
//! (`screens::draw_car_figures`, `car_caption`, `menu::load_car_background`,
//! `sound::play_car_stream`) and the carousels.

use crate::errors::{BadInput, Refused};
use crate::framework::game_read;
use crate::menu::state_store::MENU_STATE;

pub use rc2k_core::screens::car_pages::CarDescription;

const _: () = assert!(std::mem::size_of::<CarDescription>() == 0x78);

/// The carousels' entries: a table of 0x100 description pointers.
const ENTRIES: u32 = 0x100;
const SHOWN_RVA: usize = crate::framework::rva(0x755990); // dword_755990

/// The description of the entry shown (`dword_755990` through `dword_75A65C`).
pub fn shown_description() -> Result<*mut CarDescription, Refused> {
    unsafe {
        let table: u32 = MENU_STATE.carousel.entries.get();
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
