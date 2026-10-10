//! `service::apply_repairs` (0x46C14D): carries out the work chosen in the service area, on
//! leaving it: each chosen repair is shown being done, then the chosen setup is installed.
//!
//! In:  nothing
//! Out: every register preserved
//! What it does, in order (a "chosen" item is one whose flag in `dword_759B6E` is exactly 1):
//!   - `dword_75AB38` = 1;
//!   - runs of the car's dwords are zeroed for five chosen items: gearbox (0xE) ten at +0x17C,
//!     body (3) six at +0x1A4, lights (7) seven at +0x1BC, suspension (0xB) five at +0x1D8,
//!     brakes (0xD) three at +0x1EC; for the tyres (0xF) `reset_tyre_wear`, keeping the
//!     damage at +0x108;
//!   - for the items 2, 4, 6, 8, 0xA, 0xC, 0xE, 3, 5, 7, 9, 0xB, 0xD, 0xF, in that order, if
//!     chosen: frames are run, and each time 5 ms or more have passed 2 is taken off the
//!     car's damage (+0xCC + 4 * item) until it is 0 or less; then the damage is 0, the flag
//!     is cleared, the gearbox also installs the chosen gear ratio (car +0x120) and the tyres
//!     the chosen tyre (+0x10C); then 500 ms more of frames;
//!   - ride height (0x12) and suspension stiffness (0x13), if chosen: flag cleared, chosen
//!     value installed (car +0x114, +0x118), 500 ms of frames. (The original has a countdown
//!     here too, but jumps over it.)
//!   - tyre damage 0 sets bits 0x1E in the car's dword at +0x278; body damage 0 sets all its
//!     other bits and zeroes eight dwords at +0x24C;
//!   - `sleep(1000)`.
//! A frame is the game's frame function `sub_405A71` with the service area's draw callback
//! `sub_46CFD7`; neither is ported, so the Rust version calls them with `call_game`. The
//! clock is `tick_count`. They come in through `Driver`, so that the logic can be run and
//! compared without real frames.
//! Called from `sub_48E0FD` and `sub_4A16AB`. Refuses a missing car object.
//!
//! Shadow mode cannot run both for real (it takes seconds and draws): the original runs, the
//! variables are put back, the Rust version runs with a clock that jumps and no frames, the
//! results are compared and the original's are put in place. The self-test cannot run the
//! real original either (it would run frames from the console's thread): it runs a copy of
//! it whose calls to the frame function, the clock and the sleep go to stand-ins.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, restore};
use crate::service::pages::{GEAR_RATIO, RIDE_HEIGHT, SUSPENSION_STIFFNESS, TYRE};
use crate::service::reset_tyre_wear::{TYRE_DAMAGE_OFFSET, reset_tyre_wear};
use crate::service::state::{
    FLAG_TABLE_LEN, REPAIR_FLAGS_RVA, REPAIR_MARKS_RVA, current_car, repair_flag, set_repair_flag,
};
use crate::system::sleep::sleep;
use crate::system::tick_count::tick_count;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE, VirtualAlloc, VirtualFree,
};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::apply_repairs", rva(0x46C14D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const WORKING_RVA: usize = rva(0x75AB38); // dword_75AB38: set to 1 here
const START_RVA: usize = rva(0x75AB48); // dword_75AB48: the clock when a phase started
const ELAPSED_RVA: usize = rva(0x75AB4C); // dword_75AB4C
const LAST_STEP_RVA: usize = rva(0x75AB54); // dword_75AB54: `elapsed` at the last step

const FRAME_FUNCTION_RVA: usize = rva(0x405A71); // sub_405A71
const DRAW_CALLBACK_RVA: usize = rva(0x46CFD7); // sub_46CFD7

const DAMAGE_OFFSET: u32 = 0xCC; // car +0xCC + 4 * item
const BODY_ITEM: u32 = 3;
const TYRE_ITEM: u32 = 0xF;
const GEARBOX_ITEM: u32 = 0xE;
const STATE_BITS_OFFSET: u32 = 0x278;
const BODY_STATE_OFFSET: u32 = 0x24C; // eight dwords

/// (item, first offset in the car, dwords) zeroed before the repairs are shown.
const RESET_RUNS: [(u32, u32, u32); 5] = [
    (GEARBOX_ITEM, 0x17C, 10),
    (BODY_ITEM, 0x1A4, 6),
    (7, 0x1BC, 7),
    (0xB, 0x1D8, 5),
    (0xD, 0x1EC, 3),
];

/// The order the repairs are shown in.
const REPAIR_ORDER: [u32; 14] = [
    2,
    4,
    6,
    8,
    0xA,
    0xC,
    GEARBOX_ITEM,
    3,
    5,
    7,
    9,
    0xB,
    0xD,
    TYRE_ITEM,
];

const STEP_MS: i32 = 5;
const PAUSE_MS: i32 = 500;

// ---- Rust version ----

/// Where time, frames and sleep come from.
pub trait Driver {
    fn clock(&mut self) -> u32;
    fn frame(&mut self);
    fn sleep(&mut self, milliseconds: u32);
}

/// The game's own.
pub struct GameDriver;

impl Driver for GameDriver {
    fn clock(&mut self) -> u32 {
        tick_count()
    }
    fn frame(&mut self) {
        let mut regs = Regs {
            eax: game_addr(DRAW_CALLBACK_RVA) as u32,
            ..Regs::default()
        };
        unsafe { call_game(&mut regs, game_addr(FRAME_FUNCTION_RVA)) };
    }
    fn sleep(&mut self, milliseconds: u32) {
        sleep(milliseconds);
    }
}

unsafe fn field(car: u32, offset: u32) -> *mut i32 {
    car.wrapping_add(offset) as *mut i32
}

/// Frames for 500 ms.
unsafe fn pause(driver: &mut dyn Driver) {
    unsafe {
        game_write(START_RVA, driver.clock());
        loop {
            driver.frame();
            let elapsed = driver.clock().wrapping_sub(game_read(START_RVA));
            if elapsed as i32 >= PAUSE_MS {
                return;
            }
        }
    }
}

pub unsafe fn apply_repairs(driver: &mut dyn Driver) -> Result<(), Refused> {
    let Some(car) = current_car() else {
        return Err(BadInput("no current car object: the original writes through null").into());
    };
    unsafe {
        game_write(WORKING_RVA, 1u32);
        for (item, offset, dwords) in RESET_RUNS {
            if repair_flag(item) == 1 {
                for i in 0..dwords {
                    field(car, offset + 4 * i).write_unaligned(0);
                }
            }
        }
        if repair_flag(TYRE_ITEM) == 1 {
            let damage = field(car, TYRE_DAMAGE_OFFSET).read_unaligned();
            reset_tyre_wear()?;
            field(car, TYRE_DAMAGE_OFFSET).write_unaligned(damage);
        }

        for item in REPAIR_ORDER {
            if repair_flag(item) != 1 {
                continue;
            }
            let damage = field(car, DAMAGE_OFFSET + 4 * item);
            game_write(START_RVA, driver.clock());
            game_write(LAST_STEP_RVA, 0u32);
            loop {
                driver.frame();
                let elapsed = driver.clock().wrapping_sub(game_read(START_RVA));
                game_write(ELAPSED_RVA, elapsed);
                if (elapsed.wrapping_sub(game_read(LAST_STEP_RVA)) as i32) < STEP_MS {
                    continue;
                }
                game_write(LAST_STEP_RVA, elapsed);
                let left = damage.read_unaligned();
                if left <= 0 || left - 2 <= 0 {
                    break;
                }
                damage.write_unaligned(left - 2);
            }
            damage.write_unaligned(0);
            set_repair_flag(item, 0);
            if item == GEARBOX_ITEM {
                let ratio: i32 = game_read(GEAR_RATIO.change.chosen_rva);
                game_write(GEAR_RATIO.change.installed_rva, ratio);
                field(car, 0x120).write_unaligned(ratio);
            }
            if item == TYRE_ITEM {
                let tyre: i32 = game_read(TYRE.change.chosen_rva);
                game_write(TYRE.change.installed_rva, tyre);
                field(car, 0x10C).write_unaligned(tyre);
            }
            pause(driver);
        }

        for (page, offset) in [(RIDE_HEIGHT, 0x114), (SUSPENSION_STIFFNESS, 0x118)] {
            if repair_flag(page.item) != 1 {
                continue;
            }
            set_repair_flag(page.item, 0);
            let chosen: i32 = game_read(page.change.chosen_rva);
            game_write(page.change.installed_rva, chosen);
            field(car, offset).write_unaligned(chosen);
            pause(driver);
        }

        let bits = field(car, STATE_BITS_OFFSET) as *mut u32;
        if field(car, TYRE_DAMAGE_OFFSET).read_unaligned() == 0 {
            bits.write_unaligned(bits.read_unaligned() | 0x1E);
        }
        if field(car, DAMAGE_OFFSET + 4 * BODY_ITEM).read_unaligned() == 0 {
            bits.write_unaligned(bits.read_unaligned() | 0xFFFF_FFE1);
            for i in 0..8 {
                field(car, BODY_STATE_OFFSET + 4 * i).write_unaligned(0);
            }
        }
        driver.sleep(1000);
    }
    Ok(())
}

// ---- hook ----

/// What the function decides: the car from its damage values to its state bits, both flag
/// tables, the "working" switch and the block of chosen and installed setup values.
fn result_regions(car: u32) -> Vec<Region> {
    vec![
        (
            car + DAMAGE_OFFSET,
            (STATE_BITS_OFFSET + 4 - DAMAGE_OFFSET) as usize,
        ),
        (game_addr(REPAIR_FLAGS_RVA) as u32, FLAG_TABLE_LEN),
        (game_addr(REPAIR_MARKS_RVA) as u32, FLAG_TABLE_LEN),
        at(WORKING_RVA),
        (game_addr(rva(0x75AC90)) as u32, 0x2C), // dword_75AC90 .. dword_75ACB8
    ]
}

/// Its three time-keeping variables, which depend on the clock.
fn timing_regions() -> [Region; 3] {
    [at(START_RVA), at(ELAPSED_RVA), at(LAST_STEP_RVA)]
}

/// A clock that moves 5 ms at every reading, no frames, no sleep.
struct InstantDriver(u32);

impl Driver for InstantDriver {
    fn clock(&mut self) -> u32 {
        self.0 = self.0.wrapping_add(5);
        self.0
    }
    fn frame(&mut self) {}
    fn sleep(&mut self, _: u32) {}
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let Some(car) = current_car() else {
        unsafe { call_original(r) };
        return;
    };
    if !shadow() {
        if unsafe { apply_repairs(&mut GameDriver) }.is_err() {
            unsafe { call_original(r) };
        }
        return;
    }

    // The original, for real; then the Rust logic on the same starting point, without frames.
    let results = result_regions(car);
    let timing = timing_regions();
    let before = capture(&results);
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = (capture(&results), capture(&timing));

    restore(&results, &before);
    let outcome = unsafe { apply_repairs(&mut InstantDriver(0)) };
    let mine = capture(&results);
    restore(&results, &theirs.0);
    restore(&timing, &theirs.1);

    if outcome.is_err() || mine != theirs.0 || !Left::nothing().matches(r, &o) {
        HOOK.mismatch(|| {
            let first = mine.iter().zip(&theirs.0).position(|(a, b)| a != b);
            format!(
                "rust {outcome:?}; first different byte of the results: {first:x?} (0 = car +0xCC); flags before {:x?}",
                &before[results[0].1..results[0].1 + FLAG_TABLE_LEN]
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

// ---- self-test: a copy of the original with its frames, clock and sleep replaced ----

const FUNCTION_LEN: usize = 0xE8A;

/// Offsets of every `call rel32` in the function.
const CALL_OFFSETS: [usize; 98] = [
    444, 477, 503, 508, 613, 629, 634, 673, 699, 704, 809, 825, 830, 869, 895, 900, 1005, 1021,
    1026, 1065, 1091, 1096, 1201, 1217, 1222, 1261, 1287, 1292, 1397, 1413, 1418, 1457, 1483, 1488,
    1593, 1609, 1614, 1653, 1679, 1684, 1805, 1821, 1826, 1865, 1891, 1896, 2001, 2017, 2022, 2061,
    2087, 2092, 2197, 2213, 2218, 2257, 2283, 2288, 2393, 2409, 2414, 2453, 2479, 2484, 2589, 2605,
    2610, 2649, 2675, 2680, 2785, 2801, 2806, 2845, 2871, 2876, 2981, 2997, 3002, 3041, 3067, 3072,
    3193, 3209, 3214, 3255, 3281, 3286, 3407, 3423, 3428, 3469, 3495, 3500, 3621, 3637, 3642, 3715,
];

static FAKE_CLOCK: AtomicU32 = AtomicU32::new(0);
static FRAMES: AtomicU32 = AtomicU32::new(0);
static SLEPT: AtomicU32 = AtomicU32::new(0);

const CLOCK_STEP: u32 = 3;

/// Stands in for `tick_count`: eax = the fake clock, which then moves on. Like the real one
/// it changes nothing but eax and flags.
#[unsafe(naked)]
unsafe extern "C" fn fake_tick_count() {
    core::arch::naked_asm!(
        "mov eax, [{clock}]",
        "add dword ptr [{clock}], 3",
        "ret",
        clock = sym FAKE_CLOCK,
    )
}

/// Stands in for the frame function: counts the call.
#[unsafe(naked)]
unsafe extern "C" fn fake_frame() {
    core::arch::naked_asm!("inc dword ptr [{frames}]", "ret", frames = sym FRAMES)
}

/// Stands in for `sleep`: adds up what was asked for.
#[unsafe(naked)]
unsafe extern "C" fn fake_sleep() {
    core::arch::naked_asm!("add [{slept}], eax", "ret", slept = sym SLEPT)
}

/// The same stand-ins for the Rust version.
struct TestDriver;

impl Driver for TestDriver {
    fn clock(&mut self) -> u32 {
        FAKE_CLOCK.fetch_add(CLOCK_STEP, Ordering::SeqCst)
    }
    fn frame(&mut self) {
        FRAMES.fetch_add(1, Ordering::SeqCst);
    }
    fn sleep(&mut self, milliseconds: u32) {
        SLEPT.fetch_add(milliseconds, Ordering::SeqCst);
    }
}

/// An executable copy of the original with its calls redirected. Freed when dropped.
struct RedirectedCopy(*mut u8);

impl RedirectedCopy {
    unsafe fn new(original: usize) -> Result<Self, &'static str> {
        unsafe {
            let copy = VirtualAlloc(
                std::ptr::null(),
                FUNCTION_LEN,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            ) as *mut u8;
            if copy.is_null() {
                return Err("VirtualAlloc failed");
            }
            let this = Self(copy);
            std::ptr::copy_nonoverlapping(original as *const u8, copy, FUNCTION_LEN);
            // The hook's jump sits in the first bytes of the original: put the real ones back.
            std::ptr::copy_nonoverlapping(ENTRY_BYTES.as_ptr(), copy, ENTRY_BYTES.len());
            for offset in CALL_OFFSETS {
                let at = copy.add(offset);
                if at.read() != 0xE8 {
                    return Err("the function is not what the call list was made from");
                }
                let rel = (at.add(1) as *const i32).read_unaligned();
                let target = (original + offset + 5).wrapping_add(rel as usize);
                let new_target = match target.wrapping_sub(game_addr(0)) {
                    t if t == rva(0x411024) => fake_tick_count as *const () as usize,
                    t if t == rva(0x405A71) => fake_frame as *const () as usize,
                    t if t == rva(0x41100C) => fake_sleep as *const () as usize,
                    t if t == rva(0x486EDD) => target,
                    _ => return Err("a call goes somewhere unexpected"),
                };
                let new_rel = new_target.wrapping_sub(copy as usize + offset + 5) as i32;
                (at.add(1) as *mut i32).write_unaligned(new_rel);
            }
            Ok(this)
        }
    }
}

impl Drop for RedirectedCopy {
    fn drop(&mut self) {
        unsafe { VirtualFree(self.0.cast(), 0, MEM_RELEASE) };
    }
}

/// The original's first ten bytes: `pusha; mov dword_75AB38, 1`, with the address as it is
/// in the image at its preferred base (the test checks the base before using them).
const ENTRY_BYTES: [u8; 11] = [
    0x60, 0xC7, 0x05, 0x38, 0xAB, 0x75, 0x00, 0x01, 0x00, 0x00, 0x00,
];
const PREFERRED_BASE: usize = 0x40_0000;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Some(car) = current_car() else {
        return t.skip("there is no current car object");
    };
    if game_addr(0) != PREFERRED_BASE {
        return t.skip("the game is not loaded at its preferred base");
    }
    let copy = match unsafe { RedirectedCopy::new(game_addr(rva(0x46C14D))) } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };

    let mut regions = result_regions(car);
    regions.extend(timing_regions());
    let saved = capture(&regions);
    let observed = || [&FAKE_CLOCK, &FRAMES, &SLEPT].map(|counter| counter.load(Ordering::SeqCst));

    // (chosen items with their flag value, damage of every part, tyre and body damage end
    // up 0 or not through these)
    let cases: [(&[(u32, u32)], i32); 7] = [
        (&[], 40),
        (&[(2, 1)], 40),
        (
            &[(2, 1), (3, 1), (0xE, 1), (0xF, 1), (0x12, 1), (0x13, 1)],
            7,
        ),
        (&[(4, 1), (6, 2), (8, 1), (0xD, 1)], 1),
        (&[(3, 1), (5, 1), (7, 1), (9, 1), (0xB, 1), (0xF, 1)], 0),
        (&[(0xA, 1), (0xC, 1), (0x12, 2), (0x13, 1)], -3),
        (
            &[
                (2, 1),
                (3, 1),
                (4, 1),
                (5, 1),
                (6, 1),
                (7, 1),
                (8, 1),
                (9, 1),
                (0xA, 1),
                (0xB, 1),
                (0xC, 1),
                (0xD, 1),
                (0xE, 1),
                (0xF, 1),
                (0x12, 1),
                (0x13, 1),
            ],
            100,
        ),
    ];
    for (n, (chosen, damage)) in cases.into_iter().enumerate() {
        let prepare = || unsafe {
            restore(&regions, &saved);
            for item in 0..20 {
                set_repair_flag(item, 0);
            }
            for &(item, flag) in chosen {
                set_repair_flag(item, flag);
            }
            for item in 2..=0xF {
                field(car, DAMAGE_OFFSET + 4 * item).write_unaligned(damage + item as i32 % 3);
            }
            // Small numbers in what gets zeroed, so that the zeroing shows.
            for offset in (0x17C..0x20C)
                .step_by(4)
                .chain((BODY_STATE_OFFSET..0x26C).step_by(4))
            {
                field(car, offset).write_unaligned(1);
            }
            (field(car, STATE_BITS_OFFSET) as *mut u32).write_unaligned(0x100);
            game_write(WORKING_RVA, 0u32);
            for (rva, value) in [
                (GEAR_RATIO.change.chosen_rva, 3u32),
                (TYRE.change.chosen_rva, 4),
                (RIDE_HEIGHT.change.chosen_rva, 61),
                (SUSPENSION_STIFFNESS.change.chosen_rva, 62),
            ] {
                game_write(rva, value);
            }
            FAKE_CLOCK.store(1000, Ordering::SeqCst);
            FRAMES.store(0, Ordering::SeqCst);
            SLEPT.store(0, Ordering::SeqCst);
        };

        prepare();
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.0 as usize) };
        let theirs = (capture(&regions), observed());

        prepare();
        let outcome = unsafe { apply_repairs(&mut TestDriver) };
        let mine = (capture(&regions), observed());

        let preserved = ro.preserved_except(&before, 0);
        t.check(outcome.is_ok() && theirs == mine && preserved, || {
            let first = mine.0.iter().zip(&theirs.0).position(|(a, b)| a != b);
            format!(
                "case {n}: preserved={preserved} original clock/frames/slept {:?}; rust {outcome:?} {:?}; first different byte: {first:x?} (0 = car +0xCC, the car part is {:#x} long)",
                theirs.1,
                mine.1,
                regions[0].1
            )
        });
    }

    restore(&regions, &saved);
    t.finish()
}
