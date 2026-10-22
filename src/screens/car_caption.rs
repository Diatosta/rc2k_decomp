//! `screens::car_caption` (0x469A0E): the car shown's name as the caption.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The shown car's name (`CarDescription::name`, the entry `dword_755990` through
//! `dword_75A65C`) and a NUL into `g_captionText`. In four draw lists (the
//! car specification screens).
//! Refuses an entry past the records, one without its record, and a caption longer
//! than its place.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::car_description::{read, shown_description};
use crate::scene::carousel_pictures::ENTRIES_RVA;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use crate::strings::append_string_by_id::append_string_by_id;

// ---- game data ----

// dword_755990
const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText
const CAPTION_LEN: usize = 0x100;

// ---- Rust version ----

/// A string and its NUL into a place of the game's, if it fits.
pub unsafe fn put_string(id: u32, place: usize, room: usize) -> Result<(), Refused> {
    unsafe {
        let mut text = Vec::new();
        append_string_by_id(id, &mut |p, n| {
            text.extend_from_slice(std::slice::from_raw_parts(p, n))
        });
        // Converted: `rc2k_core::screens::captions::fit`.
        let text = rc2k_core::screens::captions::fit(text, room)?;
        std::ptr::copy_nonoverlapping(text.as_ptr(), game_addr(place) as *mut u8, text.len());
        Ok(())
    }
}

pub unsafe fn car_caption() -> Result<(), Refused> {
    unsafe {
        let id = read(shown_description()?).name;
        put_string(id, CAPTION_RVA, CAPTION_LEN)
    }
}

/// Carousel records of a test's: each a car and a string id, with the table of their
/// addresses put where the game keeps its own (`dword_75A65C`) until dropped.
pub struct TestRecords {
    records: Vec<[u32; 4]>,
    table: Vec<u32>,
    saved: u32,
}

impl TestRecords {
    /// One record a car of `cars`, its string one of `ids` in turn.
    pub fn new(cars: &[u32], ids: &[u32]) -> Self {
        let records: Vec<[u32; 4]> = cars
            .iter()
            .enumerate()
            .map(|(n, &car)| [car, ids[n % ids.len()], 0, 0])
            .collect();
        let table: Vec<u32> = records.iter().map(|r| r.as_ptr() as u32).collect();
        let saved = unsafe { game_read(ENTRIES_RVA) };
        unsafe { game_write(ENTRIES_RVA, table.as_ptr() as u32) };
        Self {
            records,
            table,
            saved,
        }
    }

    pub fn len(&self) -> u32 {
        self.records.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Drop for TestRecords {
    fn drop(&mut self) {
        let _ = &self.table;
        unsafe { game_write(ENTRIES_RVA, self.saved) };
    }
}

/// The cars the tests take: every car the tables have, the one that stands for
/// another among them.
pub const TEST_CARS: [u32; 12] = [0, 1, 2, 5, 0x10, 0x16, 0x17, 0x18, 0x20, 0x21, 0x22, 3];

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { car_caption() }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
