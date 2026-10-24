//! `scene::add_point_lights` (0x44F471): what the scene's lamps add to the light on an
//! object.
//!
//! In:  nothing
//! Out: nothing changed with no lamps; else ebx = their number; everything else
//!      preserved
//!
//! For each of the `dword_63DC48` lamps (0x10 bytes each from `dword_63E05C`: a point
//! of the scene, a strength, a colour; a strength of 0 is skipped): the strength to
//! `dword_639958` and the colour to `dword_62DA7C`; the lamp's distance from the
//! place being lit (`dword_636588` ..) by `scene::approximate_distance`, kept in
//! `dword_62400C`; (0x200000 less the distance) times the strength over 0x20000,
//! held to 0 .. 255, in all three colour places, each times the lamp's colour's
//! over 256; and that added to `g_lightColour` byte by byte without overflow (all
//! four bytes with MMX, the low three without, as in `scene::vertex_light`).
//! One caller, `sub_451425`.
//! Refuses more than 0x100 lamps and lamps without the scene's points.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::project_point::POINTS_RVA;
use crate::session::MACHINE;

// ---- game data ----

const LAMPS_RVA: usize = rva(0x63DC48); // dword_63DC48
const LAMP_LIST_RVA: usize = rva(0x63E05C); // dword_63E05C
const MOST: u32 = 0x100;
const STRENGTH_RVA: usize = rva(0x639958); // dword_639958
const LAMP_COLOUR_RVA: usize = rva(0x62DA7C); // dword_62DA7C
pub const LIT_PLACE_RVA: usize = rva(0x636588); // dword_636588, dword_63658C, dword_636590
pub const LIGHT_COLOUR_RVA: usize = rva(0x62DAC4); // g_lightColour

// ---- Rust version ----

// Converted: `rc2k_core::scene::lamps::add_point_lights`
pub unsafe fn add_point_lights() -> Result<(), Refused> {
    use rc2k_core::scene::lamps::{self, Lamp};
    unsafe {
        let lamps: u32 = game_read(LAMPS_RVA);
        if lamps == 0 {
            return Ok(());
        }
        let points: u32 = game_read(POINTS_RVA);
        if lamps > MOST || points == 0 {
            return Err(BadInput("more lamps than the list holds, or no points").into());
        }
        let mmx = MACHINE.has_mmx.get() != 0;
        let place: [u32; 3] = game_read(LIT_PLACE_RVA);
        let list = (0..lamps as usize).map(|n| {
            let [point, strength, colour]: [u32; 3] = game_read(LAMP_LIST_RVA + n * 0x10);
            Lamp {
                point,
                strength,
                colour,
            }
        });
        let at = |point: u32| {
            (points.wrapping_add(point.wrapping_mul(12)) as *const [u32; 3]).read_unaligned()
        };
        let out = lamps::add_point_lights(list, place, at, game_read(LIGHT_COLOUR_RVA), mmx);
        for (to, value) in [
            (STRENGTH_RVA, out.strength),
            (LAMP_COLOUR_RVA, out.colour),
            (CHECKED_RVA, out.distance),
            (LIGHT_COLOUR_RVA, out.light),
        ] {
            if let Some(v) = value {
                game_write(to, v);
            }
        }
        Ok(())
    }
}
