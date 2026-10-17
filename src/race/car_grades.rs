//! Race init's last step (`sub_43FDFE`, IDA 2026-10-17): each car block's values (floats,
//! 0 .. 100 by their use) graded against thresholds into whole numbers: `race::grade_car_values`
//! for one block, `race::grade_all_cars` for the eight.

use crate::framework::{Regs, game_read, game_write};
use crate::race::best_line::SCRATCH_RVA;
use crate::race::objects::CAR_BLOCKS_RVA;
use crate::service::harness::{Region, at};

/// (the value's offset, the grade's offset, the thresholds), in the original's order.
pub const GRADES: [(u32, u32, &[u32]); 34] = [
    (0x2B4, 0x3C0, &[50, 75, 90, 100]),
    (0x2AC, 0x3C4, &[75]),
    (0x2B8, 0x368, &[50, 75, 90, 100]),
    (0x2BC, 0x36C, &[50, 75, 90, 100]),
    (0x2C0, 0x370, &[50, 75, 90, 100]),
    (0x2C4, 0x374, &[50, 75, 90, 100]),
    (0x2C8, 0x378, &[50, 75, 90, 100]),
    (0x2CC, 0x37C, &[50, 75, 90, 100]),
    (0x2F4, 0x380, &[50, 75, 90, 100]),
    (0x30C, 0x384, &[50, 75, 90, 100]),
    (0x310, 0x388, &[50, 75, 90, 100]),
    (0x330, 0x38C, &[50, 75, 90, 100]),
    (0x350, 0x390, &[50, 75, 90, 100]),
    (0x344, 0x39C, &[50, 75, 90, 100]),
    (0x32C, 0x394, &[50, 75, 90]),
    (0x314, 0x3CC, &[100]),
    (0x318, 0x3D0, &[100]),
    (0x31C, 0x3D4, &[100]),
    (0x320, 0x3D8, &[100]),
    (0x324, 0x3DC, &[100]),
    (0x328, 0x3E0, &[100]),
    (0x2F8, 0x3E4, &[100]),
    (0x2FC, 0x3E8, &[100]),
    (0x300, 0x3EC, &[100]),
    (0x304, 0x3F0, &[100]),
    (0x364, 0x398, &[50, 75, 90]),
    (0x2D0, 0x3A0, &[100]),
    (0x2D8, 0x3A4, &[100]),
    (0x2DC, 0x3A8, &[100]),
    (0x2E0, 0x3AC, &[100]),
    (0x2E4, 0x3B0, &[100]),
    (0x2E8, 0x3B4, &[100]),
    (0x2EC, 0x3B8, &[100]),
    (0x2F0, 0x3BC, &[100]),
];

/// The grades' room in a block: +0x368 .. +0x3F4.
pub const GRADES_FROM: u32 = 0x368;
pub const GRADES_LEN: usize = 0x8C;

/// `fild t; fcomp x` then `jnb` on C0: the threshold below the value, or the value a
/// NaN (unordered sets C0).
fn below(t: u32, x: f32) -> bool {
    (t as f32) < x || x.is_nan()
}

/// `race::grade_car_values` (0x43FE24): for each of `GRADES`, the grade (+the second
/// offset) the number of its thresholds, taken in order, that lie below the block's value
/// (+the first; a NaN passes them all); each threshold tried goes through `dword_62400C`.
/// Leaves eax the last grade.
pub unsafe fn grade_car_values(entry: &Regs) -> Regs {
    let b = entry.esi;
    let mut g = *entry;
    unsafe {
        for &(from, to, thresholds) in &GRADES {
            let x = f32::from_bits(((b.wrapping_add(from)) as *const u32).read_unaligned());
            let mut grade = 0u32;
            for &t in thresholds {
                game_write(SCRATCH_RVA, t);
                if !below(t, x) {
                    break;
                }
                grade += 1;
            }
            ((b.wrapping_add(to)) as *mut u32).write_unaligned(grade);
            g.eax = grade;
        }
    }
    g
}

/// What `grade_car_values` writes for block `b`.
pub fn grade_regions(b: u32) -> Vec<Region> {
    vec![(b + GRADES_FROM, GRADES_LEN), at(SCRATCH_RVA)]
}

/// `race::grade_all_cars` (0x43FDFE): `grade_car_values` of the eight blocks
/// (`g_carBlocks`). Leaves ebx 8, esi the last block, eax its last grade.
pub unsafe fn grade_all_cars(entry: &Regs) -> Regs {
    let mut g = *entry;
    for n in 0..8u32 {
        g.ebx = n;
        g.esi = unsafe { game_read(CAR_BLOCKS_RVA + 4 * n as usize) };
        g = unsafe { grade_car_values(&g) };
    }
    g.ebx = 8;
    g
}

/// What `grade_all_cars` writes.
pub fn all_grade_regions() -> Vec<Region> {
    let mut r: Vec<Region> = (0..8u32)
        .map(|n| {
            let b: u32 = unsafe { game_read(CAR_BLOCKS_RVA + 4 * n as usize) };
            (b + GRADES_FROM, GRADES_LEN)
        })
        .collect();
    r.push(at(SCRATCH_RVA));
    r
}

/// Test values round the thresholds: below, at and just past each, NaNs, infinities, a
/// denormal, negatives.
pub const TEST_VALUES: [u32; 16] = [
    0x0000_0000, // 0
    0x4247_FFFF, // just below 50
    0x4248_0000, // 50
    0x4248_0001, // just past 50
    0x4296_0000, // 75
    0x4296_0001,
    0x42B4_0000, // 90
    0x42B4_0001,
    0x42C8_0000, // 100
    0x42C8_0001,
    0x7FC0_0000, // NaN
    0x7F80_0001, // signalling NaN
    0x7F80_0000, // infinity
    0xFF80_0000, // -infinity
    0x0000_0001, // denormal
    0xC2C8_0000, // -100
];

/// Fills block `b`'s values (+0x2AC .. +0x354, +0x364) from `TEST_VALUES` by `seed`.
pub unsafe fn fill_values(b: u32, seed: u32) {
    for (k, off) in (0x2ACu32..0x354).step_by(4).chain([0x364]).enumerate() {
        let v = TEST_VALUES[(k as u32).wrapping_mul(7).wrapping_add(seed) as usize % 16];
        unsafe { ((b + off) as *mut u32).write_unaligned(v) };
    }
}

/// The values' room, for saving.
pub fn value_region(b: u32) -> Region {
    (b + 0x2AC, 0xBC)
}
