//! `saves::gather_savegame` (0x4AB427): copies the state a saved game holds into a buffer.
//!
//! In:  edi = where it goes (0x725C bytes)
//! Out: every register preserved
//! In this order:
//!   - seven dwords: `dword_755984`, `dword_755950`, the event's stage index and section
//!     (0x755958, 0x75595C), `dword_755988`, `dword_7556D0`, `dword_75AB40`;
//!   - the four car objects `off_75647C[0 .. 4]`, 0x28C bytes each;
//!   - 180 records of 0x94 bytes from `unk_73B674`.
//!
//! Before the cars are copied it also changes them: car 0 gets `dword_7554C0` at +0x14 and
//! `dword_7554C4` at +0x274, and every car gets the 0x20 bytes its pointer at +0x18 points to
//! copied to +0x124 (a name, by its size; a reading), so that the saved copy carries them.
//! What the seven dwords and the records are has not been worked out.
//! Refuses, before anything is written: a missing car object or a car without that pointer.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::ptr::copy_nonoverlapping;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("saves::gather_savegame", rva(0x4AB427), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// The seven variables at the start, in the order they are saved.
pub const HEAD_RVAS: [usize; 7] = [
    rva(0x755984), // dword_755984
    rva(0x755950), // dword_755950
    rva(0x755958), // index: the event's stage in its section
    rva(0x75595C), // section
    rva(0x755988), // dword_755988
    rva(0x7556D0), // dword_7556D0
    rva(0x75AB40), // dword_75AB40
];
const CARS_RVA: usize = rva(0x75647C); // off_75647C[4]
pub const CARS: usize = 4;
pub const CAR_LEN: usize = 0x28C;
pub const CAR0_A_RVA: usize = rva(0x7554C0); // dword_7554C0 -> car 0 +0x14
pub const CAR0_B_RVA: usize = rva(0x7554C4); // dword_7554C4 -> car 0 +0x274
pub const NAME_PTR_OFFSET: u32 = 0x18;
pub const NAME_OFFSET: u32 = 0x124;
pub const NAME_LEN: usize = 0x20;
pub const RECORDS_RVA: usize = rva(0x73B674); // unk_73B674
pub const RECORDS_LEN: usize = 0xB4 * 0x94;

/// Bytes written to the buffer.
pub const GATHERED_LEN: usize = HEAD_RVAS.len() * 4 + CARS * CAR_LEN + RECORDS_LEN;

/// The four car objects, if every one is there and has its pointer at +0x18.
pub fn cars() -> Result<[u32; CARS], BadInput> {
    let mut cars = [0u32; CARS];
    for (n, car) in cars.iter_mut().enumerate() {
        *car = unsafe { ((game_addr(CARS_RVA) as *const u32).add(n)).read_unaligned() };
        if *car == 0 {
            return Err(BadInput(
                "a car object is missing: the original reads through null",
            ));
        }
        if unsafe { ((*car + NAME_PTR_OFFSET) as *const u32).read_unaligned() } == 0 {
            return Err(BadInput(
                "a car has no pointer at +0x18: the original reads through null",
            ));
        }
    }
    Ok(cars)
}

// ---- Rust version ----

pub unsafe fn gather_savegame(mut cursor: *mut u8) -> Result<(), BadInput> {
    let cars = cars()?;
    unsafe {
        for at in HEAD_RVAS {
            (cursor as *mut u32).write_unaligned(game_read(at));
            cursor = cursor.add(4);
        }
        ((cars[0] + 0x14) as *mut u32).write_unaligned(game_read(CAR0_A_RVA));
        ((cars[0] + 0x274) as *mut u32).write_unaligned(game_read(CAR0_B_RVA));
        for car in cars {
            let name = ((car + NAME_PTR_OFFSET) as *const u32).read_unaligned();
            std::ptr::copy(name as *const u8, (car + NAME_OFFSET) as *mut u8, NAME_LEN);
            copy_nonoverlapping(car as *const u8, cursor, CAR_LEN);
            cursor = cursor.add(CAR_LEN);
        }
        copy_nonoverlapping(game_addr(RECORDS_RVA) as *const u8, cursor, RECORDS_LEN);
    }
    Ok(())
}

// ---- hook ----

/// The buffer, and what is changed in the cars.
fn regions(buffer: u32) -> Vec<Region> {
    let mut regions = vec![(buffer, GATHERED_LEN + 0x10)];
    if let Ok(cars) = cars() {
        regions.push((cars[0] + 0x14, 4));
        regions.push((cars[0] + 0x274, 4));
        regions.extend(cars.map(|car| (car + NAME_OFFSET, NAME_LEN)));
    }
    regions
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { gather_savegame(r.edi as *mut u8) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    // The buffer is the caller's and exactly as long as what is written.
    let mut regions = regions(r.edi);
    regions[0].1 = GATHERED_LEN;
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test: the game as it is, into a buffer of its own ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(cars) = cars() else {
        return t.skip("the car objects are not there");
    };
    let mut buffer = vec![0u8; GATHERED_LEN + 0x10];
    let regions = regions(buffer.as_mut_ptr() as u32);
    let saved = capture(&regions[1..]);
    let before = Regs {
        edi: buffer.as_mut_ptr() as u32,
        ..Regs::SENTINEL
    };
    let buffer_ptr = buffer.as_mut_ptr();
    for fill in [0xAAu8, 0x55] {
        check_state(
            &mut t,
            &format!("fill={fill:#x}"),
            call_original,
            before,
            &|| unsafe {
                buffer_ptr.write_bytes(fill, GATHERED_LEN + 0x10);
                // What the function rewrites in the cars gets a different value first.
                ((cars[0] + 0x14) as *mut u32).write_unaligned(0x5EED_0001);
                ((cars[0] + 0x274) as *mut u32).write_unaligned(0x5EED_0002);
                for car in cars {
                    ((car + NAME_OFFSET) as *mut u8).write_bytes(fill, NAME_LEN);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&regions[1..], &saved);
    t.finish()
}
