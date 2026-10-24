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

use crate::framework::{game_ptr, game_read, rva};
use crate::scene::light_tables::{
    CAR_LIGHT_TABLE_RVA, LAMP_TABLE_A_RVA, LAMP_TABLE_B_RVA, LAMP_TABLE_C_RVA, LIGHT_TABLE_RVA,
    fill_light_ramp, fill_light_scale,
};

// ---- game data ----

const OTHER_WAY_RVA: usize = rva(0x621480); // dword_621480
const OTHER_WAY_B_RVA: usize = rva(0x60931C); // dword_60931C
const PLAIN_RVA: usize = rva(0x62DB10); // unk_62DB10: two colours
const SCENE_RVA: usize = rva(0x6092B4); // dword_6092B4: two colours
/// The four lamp tables and, the other way, their colours.
const LAMPS: [(usize, usize); 4] = [
    (LAMP_TABLE_A_RVA, rva(0x60A54C)),
    (LAMP_TABLE_B_RVA, rva(0x60A558)),
    (CAR_LIGHT_TABLE_RVA, rva(0x60A564)),
    (LAMP_TABLE_C_RVA, rva(0x60A570)),
];

// ---- Rust version ----

// Converted: `rc2k_core::scene::lamps::plain_light`
pub unsafe fn fill_light_tables() {
    unsafe {
        let plain = rc2k_core::scene::lamps::plain_light(game_read(OTHER_WAY_RVA), || {
            game_read(OTHER_WAY_B_RVA)
        });
        let ramp = if plain { PLAIN_RVA } else { SCENE_RVA };
        fill_light_ramp(game_ptr(ramp), game_ptr(LIGHT_TABLE_RVA));
        for (table, colour) in LAMPS {
            let colour = if plain { PLAIN_RVA } else { colour };
            fill_light_scale(game_ptr(colour), game_ptr(table));
        }
    }
}
