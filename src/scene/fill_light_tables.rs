//! `scene::fill_light_tables` (0x44F0AB): the five tables of colours the lighting
//! picks from.
//!
//! In:  nothing
//! Out: eax, ebx, ecx, edx, ebp as `scene::fill_light_scale` leaves them, esi = the
//!      last colour, edi = the last table
//!
//! With `dword_621480` and `dword_60931C` both 0 (the plain way): `g_lightTable` a ramp between
//! the two colours at `unk_62DB10`, and the four lamp tables (`dword_635518`, `dword_635918`,
//! `g_carLightTable`, `unk_636118`) that first colour scaled. Else: `g_lightTable` a ramp
//! between the two colours at `dword_6092B4` (the dark and the lit colour of the scene), and
//! the four lamp tables the colours at `unk_60A54C`, `unk_60A558`, `dword_60A564`,
//! `dword_60A570` scaled. Four callers.

use crate::framework::game_ptr;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::scene::light_tables::{
    CAR_LIGHT_TABLE_RVA, LAMP_TABLE_A_RVA, LAMP_TABLE_B_RVA, LAMP_TABLE_C_RVA, LIGHT_TABLE_RVA,
    fill_light_ramp, fill_light_scale,
};
use crate::scene::light_work_store::LIGHT_WORK;

// ---- Rust version ----

// Converted: `rc2k_core::scene::lamps::plain_light`
pub unsafe fn fill_light_tables() {
    unsafe {
        let plain = rc2k_core::scene::lamps::plain_light(FRAME_SWITCHES.other_way.bits(), || {
            STAGE_PREFS.lit_b.bits()
        });
        // `unk_62DB10`: two colours; `dword_6092B4`: two colours.
        let plain_colours = LIGHT_WORK.plain_light_colours[0].addr();
        let ramp = if plain {
            plain_colours
        } else {
            STAGE_PREFS.dark_colour[0].addr()
        };
        fill_light_ramp(ramp as *const [u32; 6], game_ptr(LIGHT_TABLE_RVA));
        // The four lamp tables and, the other way, their colours.
        let lamps = [
            (LAMP_TABLE_A_RVA, STAGE_PREFS.lamp_table_a[0].addr()),
            (LAMP_TABLE_B_RVA, STAGE_PREFS.lamp_table_b[0].addr()),
            (CAR_LIGHT_TABLE_RVA, STAGE_PREFS.car_light_table[0].addr()),
            (LAMP_TABLE_C_RVA, STAGE_PREFS.lamp_table_c.addr()),
        ];
        for (table, colour) in lamps {
            let colour = if plain { plain_colours } else { colour };
            fill_light_scale(colour as *const [u32; 3], game_ptr(table));
        }
    }
}
