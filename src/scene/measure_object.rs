//! `scene::measure_object` (0x4222B8): finds the middle of the object being loaded
//! and moves its points so that the middle is at 0.
//!
//! In:  nothing (`dword_634FEC` the object's slot)
//! Out: ebx = the slot, ecx = 0, esi = past the object's points, eax = the middle's z;
//!      for an object of no points: ebx = the slot, ecx = 0, esi = its points' place
//!
//! The middle (`dword_634B70` ..) = 0; the lowest of each of x, y, z (`dword_634B88`
//! ..) from 0x640000 down and the highest (`dword_634B94` ..) from -0x640000 up over
//! the slot's points; half the extent of each (`dword_634B7C` ..) = (highest - lowest)
//! halved, the middle = that plus the lowest; then the middle is taken off every
//! point.
//! Eleven call sites in `scene::load_object` (the parts of a car).
//! Refuses a slot past the tables and points without their memory.
//! Self-test: 24 sets of points by lot in slot 0xF3 of the game's tables.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBX, ECX, ESI, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::scene::objects::{
    FIRST_POINT_RVA, POINT_COUNTS_RVA, POINTS_RVA, SLOT_RVA, SLOTS, of_slot, slot_entry,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::measure_object", rva(0x4222B8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// `dword_634B70` .. `dword_634B9C`: the middle, half the extent, the lowest, the
/// highest, three dwords each.
pub const MEASURES_RVA: usize = rva(0x634B70);
/// More points than this are not an object's (the test and the regions' guard).
const MOST: u32 = 0x10_0000;

// ---- Rust version ----

unsafe fn points_of(slot: u32) -> Result<(usize, u32), BadInput> {
    unsafe {
        if slot >= SLOTS {
            return Err(BadInput("an object's slot past the tables"));
        }
        let count = of_slot(POINT_COUNTS_RVA, slot);
        let memory: u32 = game_read(POINTS_RVA);
        if count > MOST || (count != 0 && memory == 0) {
            return Err(BadInput("points without their memory, or too many"));
        }
        let first = of_slot(FIRST_POINT_RVA, slot);
        Ok((memory.wrapping_add(first.wrapping_mul(12)) as usize, count))
    }
}

// Converted: `rc2k_core::scene::objects::measure`
pub unsafe fn measure_object() -> Result<Left, BadInput> {
    use rc2k_core::scene::objects::{HIGHEST_START, LOWEST_START, measure};
    unsafe {
        let slot: u32 = game_read(SLOT_RVA);
        let (at, count) = points_of(slot)?;
        let left = Left::nothing().with(EBX, slot).with(ECX, 0);
        let points: &mut [[u32; 3]] = if count == 0 {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(at as *mut [u32; 3], count as usize)
        };
        let Some(m) = measure(points) else {
            // The half extents are not touched for an object of no points.
            game_write(MEASURES_RVA, [0u32; 3]);
            game_write(MEASURES_RVA + 0x18, [[LOWEST_START; 3], [HIGHEST_START; 3]]);
            return Ok(left.with(ESI, at as u32));
        };
        game_write(MEASURES_RVA, [m.middle, m.half, m.lowest, m.highest]);
        Ok(left
            .with(EAX, m.middle[2])
            .with(ESI, (at + count as usize * 12) as u32))
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![(game_addr(MEASURES_RVA) as u32, 0x30)];
    if let Ok((at, count)) = unsafe { points_of(game_read(SLOT_RVA)) }
        && count != 0
    {
        regions.push((at as u32, count as usize * 12));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { measure_object() }?)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    const SLOT: u32 = 0xF3;
    let inputs = [
        at(SLOT_RVA),
        at(POINTS_RVA),
        slot_entry(FIRST_POINT_RVA, SLOT),
        slot_entry(POINT_COUNTS_RVA, SLOT),
        (game_addr(MEASURES_RVA) as u32, 0x30),
    ];
    let saved = capture(&inputs);
    let mut seed = 0x0B0E_C701u32;
    let mut lot = |below: u32| {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) % below
    };
    for case in 0..24u32 {
        let count = [0u32, 1, 2, 7, 40][case as usize % 5];
        // Three points before the object's, which must stay.
        let mut points = vec![[0xAAAA_AAAAu32; 3]; count as usize + 4];
        for point in &mut points[3..3 + count as usize] {
            for part in point.iter_mut() {
                *part = match lot(9) {
                    0 => 0x7FFF_FFFF,
                    1 => 0x8000_0000,
                    2 => 0x0064_0000,
                    3 => 0xFF9C_0000,
                    _ => lot(0x80_0000).wrapping_sub(0x40_0000),
                };
            }
        }
        let memory = std::cell::RefCell::new(points.clone());
        let regions = [
            (game_addr(MEASURES_RVA) as u32, 0x30),
            (memory.borrow().as_ptr() as u32, points.len() * 12),
        ];
        check_state(
            &mut t,
            &format!(
                "case {case}: {count} points {:x?}",
                &points[3..3 + count as usize]
            ),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                memory.borrow_mut().copy_from_slice(&points);
                game_write(MEASURES_RVA, [0xAAAA_AAAAu32; 12]);
                game_write(SLOT_RVA, SLOT);
                game_write(POINTS_RVA, memory.borrow().as_ptr() as u32);
                game_write(FIRST_POINT_RVA + SLOT as usize * 4, 3u32);
                game_write(POINT_COUNTS_RVA + SLOT as usize * 4, count);
            },
            &regions,
            &rust,
        );
    }
    restore(&inputs, &saved);
    t.finish()
}
