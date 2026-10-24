//! `screens::draw_inspection_status` (0x4A6B54): works out the inspection status, a percentage,
//! from the current car's thirteen damage values and draws it as a bar at the bottom of the
//! service area, with a sprite to its left and a label at each end.
//!
//! In:  ebx, passed on as the palette of the sprite at the bar's left
//! Out: what the last `draw_string_by_id` leaves (carry, eax = 0, ebx, ecx = 652, edx);
//!      edi = the player; esi = the address of the repair flag of the last damaged part
//!      looked at, or unchanged; ebp preserved
//! The percentage is 100 minus the weights of the parts that are damaged and not chosen for
//! repair. For menu items 2, 3, 4, 5 and 7 .. 15: the part counts if the car's value at
//! +0xCC + 4 * item is at least `dword_75ABD0[item]` and the item's repair flag is 0; its
//! weight is `dword_75AC40[item]`. The result is left in `dword_75AC80`.
//! The items are the fourteen parts of the repair section without the turbo (item 6).
//!
//! Screen: seen in a screenshot of the championship's service area (2026-10-10): "ESTADO DE INSPECÇÃO" with a
//! red to green bar between "REPROV." (string 0x4AE) and "PASSAR" (0x4AF): fail .. pass. The
//! "ESTADO DE INSPECÇÃO" label itself is drawn by `draw_service_area`. Named
//! `draw_car_condition` before the screenshot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, rva};
use crate::service::state::{current_player, repair_flag};
use crate::text::draw_text::Canvas;

// ---- game data ----

const THRESHOLDS_RVA: usize = rva(0x75ABD0); // dword_75ABD0[item] (items 2 .. 15 are used)
const WEIGHTS_RVA: usize = rva(0x75AC40); // dword_75AC40[item]
// dword_75AC80
// dword_7559EC: a work variable, here the weights' sum
const CAR_DAMAGE_OFFSET: u32 = 0xCC; // car +0xCC + 4 * item

unsafe fn table(table_rva: usize, item: u32) -> i32 {
    unsafe { ((game_addr(table_rva) as *const i32).add(item as usize)).read_unaligned() }
}

// ---- Rust version ----

/// The percentage and the item of the last damaged part looked at (its repair flag's address
/// is what the original leaves in esi). Converted: `rc2k_core::screens::inspection::status`.
pub unsafe fn inspection_status(player: u32) -> (i32, Option<u32>) {
    use rc2k_core::screens::inspection::{PARTS, Part, status};
    let parts: [Part; 13] = std::array::from_fn(|n| {
        let item = PARTS[n];
        unsafe {
            Part {
                damage: (player.wrapping_add(CAR_DAMAGE_OFFSET + 4 * item) as *const i32)
                    .read_unaligned(),
                threshold: table(THRESHOLDS_RVA, item),
                weight: table(WEIGHTS_RVA, item),
                repairing: repair_flag(item) != 0,
            }
        }
    });
    let (condition, last) = status(&parts);
    (condition, last.map(|n| PARTS[n]))
}

/// `icon_palette` is what the caller has in ebx. Refuses a missing player. The carry of
/// the text (too wide): what the one caller, `screens::draw_service_area`, reads of what
/// the original leaves.
pub unsafe fn draw_inspection_status(
    icon_palette: u32,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    unsafe {
        let Some(player) = current_player() else {
            return Err(
                BadInput("no current player object: the original reads through null").into(),
            );
        };
        let (condition, _) = inspection_status(player);
        // Converted: `rc2k_core::screens::inspection::inspection_status`.
        let out = rc2k_core::screens::inspection::inspection_status(
            &mut crate::screens::game_screen::GameScreen::<
                rc2k_core::screens::inspection::Font,
                rc2k_core::screens::inspection::Work,
            >::new(canvas),
            condition,
            icon_palette,
        )?;
        Ok(out.too_wide)
    }
}
