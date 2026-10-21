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
use crate::framework::{
    EBX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::project_point::POINTS_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::add_point_lights", rva(0x44F471), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const LAMPS_RVA: usize = rva(0x63DC48); // dword_63DC48
const LAMP_LIST_RVA: usize = rva(0x63E05C); // dword_63E05C
const MOST: u32 = 0x100;
const STRENGTH_RVA: usize = rva(0x639958); // dword_639958
const LAMP_COLOUR_RVA: usize = rva(0x62DA7C); // dword_62DA7C
pub const LIT_PLACE_RVA: usize = rva(0x636588); // dword_636588, dword_63658C, dword_636590
pub const LIGHT_COLOUR_RVA: usize = rva(0x62DAC4); // g_lightColour
pub const MMX_RVA: usize = rva(0x5183A8); // g_hasMmx

// ---- Rust version ----

// Converted: `rc2k_core::scene::lamps::add_point_lights`
pub unsafe fn add_point_lights() -> Result<Left, Refused> {
    use rc2k_core::scene::lamps::{self, Lamp};
    unsafe {
        let lamps: u32 = game_read(LAMPS_RVA);
        if lamps == 0 {
            return Ok(Left::nothing());
        }
        let points: u32 = game_read(POINTS_RVA);
        if lamps > MOST || points == 0 {
            return Err(BadInput("more lamps than the list holds, or no points").into());
        }
        let mmx = game_read::<u32>(MMX_RVA) != 0;
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
        Ok(Left::nothing().with(EBX, lamps))
    }
}

// ---- hook ----

fn regions() -> [Region; 4] {
    [
        at(STRENGTH_RVA),
        at(LAMP_COLOUR_RVA),
        at(CHECKED_RVA),
        at(LIGHT_COLOUR_RVA),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { add_point_lights() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let inputs = [
        at(LAMPS_RVA),
        (game_addr(LAMP_LIST_RVA) as u32, 4 * 0x10),
        at(POINTS_RVA),
        (game_addr(LIT_PLACE_RVA) as u32, 12),
        at(MMX_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let points: [[i32; 3]; 4] = [
        [0, 0, 0],
        [0x10_0000, -0x8_0000, 0x4_0000],
        [-0x30_0000, 0x1234, 0x7FFF_FFFF],
        [100, 200, 300],
    ];
    let lamps: [[u32; 4]; 4] = [
        [1, 0x100, 0x00FF_8040, 0xAAAA_AAAA],
        [3, 0, 0x00FF_FFFF, 0xAAAA_AAAA],
        [2, 0xFFFF_FF00, 0x0012_3456, 0xAAAA_AAAA],
        [0, 0x4000, 0xFFC0_E0FF, 0xAAAA_AAAA],
    ];
    for count in 0..=4u32 {
        for place in [
            [0i32, 0, 0],
            [0x10_0000, 0, 0x4_0000],
            [-0x100, 0x7_0000, 0x12_0000],
        ] {
            for (mmx, colour) in [(0u32, 0u32), (1, 0), (0, 0x8010_F0FE), (1, 0x8010_F0FE)] {
                check_state(
                    &mut t,
                    &format!("{count} lamps, place {place:x?}, mmx {mmx}, colour {colour:#x}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        restore(&regions, &vec![0xAA; saved.0.len()]);
                        game_write(LAMPS_RVA, count);
                        game_write(LAMP_LIST_RVA, lamps);
                        game_write(POINTS_RVA, points.as_ptr() as u32);
                        game_write(LIT_PLACE_RVA, place);
                        game_write(MMX_RVA, mmx);
                        game_write(LIGHT_COLOUR_RVA, colour);
                    },
                    &regions,
                    &rust,
                );
            }
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
