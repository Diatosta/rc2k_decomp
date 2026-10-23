//! The four item actions that open an edit page of the service area that costs service
//! time (`service::begin_gear_ratio_edit` .. `begin_suspension_stiffness_edit`, 0x46B674,
//! 0x46B6BD, 0x46B72E, 0x46B777): if the highlighted item is chosen already (its flag 1),
//! or the service time still covers the chosen repairs plus this item's time
//! (`budget - (total + time)`, signed, at least 0), the value at the edit's start is the
//! chosen one, edit mode flips (`xor 1`) and the edit page is the item's. Else nothing.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Refuses (before anything) a highlighted item past the flag table (the original reads
//! past it). Self-test: the original and the Rust version on the variables, each flag,
//! budgets around the boundary, edit mode 0 and 1, put back.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::service::state::{
    EDIT_MODE_RVA, EDIT_PAGE_RVA, REPAIR_TOTAL_RVA, TIME_BUDGET_RVA, flagged_item, repair_flag,
};

/// One page's own variables.
#[derive(Clone, Copy)]
pub struct Edit {
    /// The page number written to `dword_75AAA0`.
    pub page: u32,
    /// The item's service time (`dword_75AB90` ..).
    pub time_rva: usize,
    /// The chosen value, copied ...
    pub chosen_rva: usize,
    /// ... to the value at the edit's start.
    pub start_rva: usize,
}

/// The Rust version (see the header).
pub unsafe fn begin_edit(e: Edit) -> Result<(), Refused> {
    let item = flagged_item()?;
    unsafe {
        // Converted: `rc2k_core::service::repairs::may_edit`.
        if rc2k_core::service::repairs::may_edit(
            repair_flag(item),
            game_read(REPAIR_TOTAL_RVA),
            game_read(e.time_rva),
            game_read(TIME_BUDGET_RVA),
        ) {
            game_write(e.start_rva, game_read::<u32>(e.chosen_rva));
            game_write(EDIT_MODE_RVA, game_read::<u32>(EDIT_MODE_RVA) ^ 1);
            game_write(EDIT_PAGE_RVA, e.page);
        }
    }
    Ok(())
}

pub const GEAR_RATIO: Edit = Edit {
    page: 1,
    time_rva: rva(0x75AB90),
    chosen_rva: rva(0x75AC90),
    start_rva: rva(0x75AC98),
};
pub const TYRE: Edit = Edit {
    page: 2,
    time_rva: rva(0x75AB94),
    chosen_rva: rva(0x75AC9C),
    start_rva: rva(0x75ACA4),
};
pub const RIDE_HEIGHT: Edit = Edit {
    page: 5,
    time_rva: rva(0x75AB98),
    chosen_rva: rva(0x75ACA8),
    start_rva: rva(0x75ACB0),
};
pub const SUSPENSION_STIFFNESS: Edit = Edit {
    page: 6,
    time_rva: rva(0x75AB9C),
    chosen_rva: rva(0x75ACB4),
    start_rva: rva(0x75ACBC),
};
