//! Race init's last step (`sub_43FDFE`, IDA 2026-10-17): each car block's values (floats,
//! 0 .. 100 by their use) graded against thresholds into whole numbers: `race::grade_car_values`
//! for one block, `race::grade_all_cars` for the eight.

use crate::scene::draw_store::DRAW;

pub use rc2k_core::race::cars::GRADES;

/// `race::grade_car_values` (0x43FE24): for each of `GRADES`, the grade (+the second
/// offset) the number of its thresholds, taken in order, that lie below the block's value
/// (+the first; a NaN passes them all); each threshold tried goes through `dword_62400C`.
/// The original leaves eax the last grade; its one caller reads nothing of it.
pub unsafe fn grade_car_values(block: u32) {
    // Converted: `rc2k_core::race::cars::grade`; `dword_62400C` gets the last threshold
    // tried (the ones before it are written over at once).
    unsafe {
        for &(from, to, thresholds) in &GRADES {
            let x = f32::from_bits(((block.wrapping_add(from)) as *const u32).read_unaligned());
            let (grade, tried) = rc2k_core::race::cars::grade(x, thresholds);
            DRAW.scratch.set_bits(tried);
            ((block.wrapping_add(to)) as *mut u32).write_unaligned(grade);
        }
    }
}

/// `race::grade_all_cars` (0x43FDFE): `grade_car_values` of the eight blocks
/// (`g_carBlocks`). The registers it leaves (ebx 8, esi the last block, eax its last
/// grade) are read by no caller (race init's one call).
pub unsafe fn grade_all_cars() {
    for n in 0..8usize {
        unsafe { grade_car_values(crate::race::work_store::RACE_WORK.car_block(n)) };
    }
}
