//! Race init's last step (`sub_43FDFE`, IDA 2026-10-17): each car block's values (floats,
//! 0 .. 100 by their use) graded against thresholds into whole numbers: `race::grade_car_values`
//! for one block, `race::grade_all_cars` for the eight.

use crate::framework::{Regs, game_read, game_write};
use crate::race::best_line::SCRATCH_RVA;
use crate::race::objects::CAR_BLOCKS_RVA;

pub use rc2k_core::race::cars::GRADES;

/// `race::grade_car_values` (0x43FE24): for each of `GRADES`, the grade (+the second
/// offset) the number of its thresholds, taken in order, that lie below the block's value
/// (+the first; a NaN passes them all); each threshold tried goes through `dword_62400C`.
/// Leaves eax the last grade.
pub unsafe fn grade_car_values(entry: &Regs) -> Regs {
    // Converted: `rc2k_core::race::cars::grade`; `dword_62400C` gets the last threshold
    // tried (the ones before it are written over at once).
    let b = entry.esi;
    let mut g = *entry;
    unsafe {
        for &(from, to, thresholds) in &GRADES {
            let x = f32::from_bits(((b.wrapping_add(from)) as *const u32).read_unaligned());
            let (grade, tried) = rc2k_core::race::cars::grade(x, thresholds);
            game_write(SCRATCH_RVA, tried);
            ((b.wrapping_add(to)) as *mut u32).write_unaligned(grade);
            g.eax = grade;
        }
    }
    g
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
