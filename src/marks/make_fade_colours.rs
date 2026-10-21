//! `marks::make_fade_colours` (0x4C37EF): 0x11 colours from `dword_6415E4` toward white:
//! each component's inverse (0xFF less it) times a step that grows from 0 by
//! `flt_7651C4` (`dword_8DB4A8`, a float), rounded, made a pixel (alpha 0xFF) and added to
//! the colour (as one dword, carries crossing components), into `dword_8DB464`.
//!
//! In:  nothing
//! Out: eax the last entry, ebx, ecx the green and blue inverses, esi past the entries
//!
//! One caller, `sub_4C25F6`. Self-test: the original and the Rust version on several
//! colours and steps, at the test precisions; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, ROUNDED, Tolerance, at_test_precisions, check_state_floats,
    dispatch_state_floats,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::make_fade_colours", rva(0x4C37EF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const COLOUR_RVA: usize = rva(0x6415E4); // dword_6415E4
const FADES_RVA: usize = rva(0x8DB464); // g_marksFadeColours: 0x11 dwords
const FADES: u32 = 0x11;
const STEP_RVA: usize = rva(0x8DB4A8); // dword_8DB4A8 (a float)
const INVERSES_RVA: usize = rva(0x8DB4AC); // three dwords
const SCALED_RVA: usize = rva(0x8DB4B8); // three dwords
const WORK_LEN: usize = 0x60; // the entries to the scaled three
const GROWTH_RVA: usize = rva(0x7651C4); // flt_7651C4

/// The Rust version (see the header).
pub unsafe fn make_fade_colours(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let colour: u32 = game_read(COLOUR_RVA);
        g.eax = ((colour >> 16) & 0xFF) ^ 0xFF;
        g.ebx = ((colour >> 8) & 0xFF) ^ 0xFF;
        g.ecx = (colour & 0xFF) ^ 0xFF;
        game_write(INVERSES_RVA, [g.eax, g.ebx, g.ecx]);
        game_write(STEP_RVA, 0u32);
        g.esi = game_addr(FADES_RVA) as u32;
        for _ in 0..FADES {
            let step: u32 = game_read(STEP_RVA);
            let inverses: [i32; 3] = game_read(INVERSES_RVA);
            let scaled = inverses.map(|v| x87::int_times_to_int(v, step));
            game_write(SCALED_RVA, scaled);
            let [r, gr, b] = scaled.map(|v| v as u32 & 0xFF);
            g.eax = (0xFF00_0000 | r << 16 | gr << 8 | b).wrapping_add(game_read(COLOUR_RVA));
            (g.esi as *mut u32).write_unaligned(g.eax);
            g.esi += 4;
            game_write(STEP_RVA, x87::add(step, game_read(GROWTH_RVA)));
        }
    }
    g
}

fn regions() -> Vec<Region> {
    vec![(game_addr(FADES_RVA) as u32, WORK_LEN)]
}

/// The converted `make_fade_colours` (plain floats): the hook's. The x87 version above
/// stays for `marks::run_snow` (unconverted).
// Converted: `rc2k_core::marks::fades`
pub unsafe fn make_fade_colours_plain(entry: &Regs) -> Regs {
    use rc2k_core::marks::fades;
    let mut g = *entry;
    unsafe {
        let colour: u32 = game_read(COLOUR_RVA);
        let inverses = fades::inverses(colour);
        [g.eax, g.ebx, g.ecx] = inverses;
        game_write(INVERSES_RVA, inverses);
        let f = fades::fades(colour, f32::from_bits(game_read(GROWTH_RVA)));
        game_write(FADES_RVA, f.colours);
        game_write(SCALED_RVA, f.scaled);
        game_write(STEP_RVA, f.step.to_bits());
        g.eax = f.colours[FADES as usize - 1];
        g.esi = game_addr(FADES_RVA) as u32 + 4 * FADES;
    }
    g
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { make_fade_colours_plain(r) }))
}

/// The float places: the step, and the scaled three (`fistp`).
fn float_places() -> [(Region, Tolerance); 2] {
    [
        (at(STEP_RVA), FLOAT),
        ((game_addr(SCALED_RVA) as u32, 12), ROUNDED),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places();
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let inputs: Vec<Region> = regions
        .iter()
        .copied()
        .chain([COLOUR_RVA, GROWTH_RVA].map(at))
        .collect();
    let saved = capture(&inputs);
    let game_growth: u32 = unsafe { game_read(GROWTH_RVA) };
    let places = float_places();
    let tolerant = Floats {
        memory: &places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for colour in [0u32, 0xFFFF_FFFF, 0x0080_4020, 0xFF12_34FE, 0x00FF_0001] {
            for growth in [
                game_growth,
                0.0625f32.to_bits(),
                0.07f32.to_bits(),
                (-0.1f32).to_bits(),
            ] {
                check_state_floats(
                    &mut t,
                    "marks::make_fade_colours",
                    &format!("control word {word:#x}, colour {colour:#x}, growth {growth:#x}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        restore(&inputs, &saved);
                        std::ptr::write_bytes(game_addr(FADES_RVA) as *mut u8, 0xAA, WORK_LEN);
                        game_write(COLOUR_RVA, colour);
                        game_write(GROWTH_RVA, growth);
                    },
                    &regions,
                    held,
                    &rust,
                );
            }
        }
    });
    restore(&inputs, &saved);
    t.finish()
}
