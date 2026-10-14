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
//! Self-test: records of the test's, twelve cars.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::car_description::{read, shown_description};
use crate::scene::carousel_pictures::ENTRIES_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::string_table::blob;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::car_caption", rva(0x469A0E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const CAR_RVA: usize = rva(0x755990); // dword_755990
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
        if text.len() >= room {
            return Err(BadInput("a string longer than its place").into());
        }
        text.push(0);
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

// ---- hook ----

fn regions() -> [Region; 1] {
    [(game_addr(CAPTION_RVA) as u32, CAPTION_LEN)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { car_caption() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if blob().is_null() {
        return t.skip("string table not loaded");
    }
    let inputs = [at(CAR_RVA)];
    let saved = (capture(&regions()), capture(&inputs));
    let records = TestRecords::new(&TEST_CARS, &[0x249, 0x24A, 0x24B, 0x24C, 0x537]);
    for car in 0..records.len() {
        check_state(
            &mut t,
            &format!("entry={car}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(CAR_RVA, car);
                restore(&regions(), &[0xAAu8; CAPTION_LEN]);
            },
            &regions(),
            &rust,
        );
    }
    drop(records);
    restore(&regions(), &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
