//! `scene::fill_light_tables` (0x44F0AB): the five tables of colours the lighting
//! picks from.
//!
//! In:  nothing
//! Out: eax, ebx, ecx, edx, ebp as `scene::fill_light_scale` leaves them, esi = the
//!      last colour, edi = the last table
//!
//! With `dword_621480` and `dword_60931C` both 0 (the plain way): `g_lightTable` a
//! ramp between the two colours at `unk_62DB10`, and the four lamp tables
//! (`dword_635518`, `dword_635918`, `g_carLightTable`, `unk_636118`) that first
//! colour scaled. Else: `g_lightTable` a ramp between the two colours at
//! `dword_6092B4` (the dark and the lit colour of the scene), and the four lamp tables
//! the colours at `unk_60A54C`, `unk_60A558`, `dword_60A564`, `dword_60A570` scaled.
//! Four callers.
//! Self-test: 24 settings by lot on the game's tables, put back afterwards.

use crate::errors::Refused;
use crate::framework::{
    EDI, ESI, RawHook, Regs, SelfTest, game_addr, game_ptr, game_read, game_write, hook_asm, rva,
};
use crate::scene::light_tables::{
    CAR_LIGHT_TABLE_RVA, LAMP_TABLE_A_RVA, LAMP_TABLE_B_RVA, LAMP_TABLE_C_RVA, LIGHT_TABLE_RVA,
    fill_light_ramp, fill_light_scale, table, test_colours,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::fill_light_tables", rva(0x44F0AB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

pub unsafe fn fill_light_tables() -> Left {
    unsafe {
        let plain = game_read::<u32>(OTHER_WAY_RVA) == 0 && game_read::<u32>(OTHER_WAY_B_RVA) == 0;
        let ramp = if plain { PLAIN_RVA } else { SCENE_RVA };
        fill_light_ramp(game_ptr(ramp), game_ptr(LIGHT_TABLE_RVA));
        let mut left = Left::nothing();
        for (table, colour) in LAMPS {
            let colour = if plain { PLAIN_RVA } else { colour };
            left = fill_light_scale(game_ptr(colour), game_ptr(table))
                .with(ESI, game_addr(colour) as u32)
                .with(EDI, game_addr(table) as u32);
        }
        left
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![table(LIGHT_TABLE_RVA)];
    regions.extend(LAMPS.iter().map(|&(lamp, _)| table(lamp)));
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { fill_light_tables() })
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
    let inputs: Vec<Region> = vec![
        at(OTHER_WAY_RVA),
        at(OTHER_WAY_B_RVA),
        (game_addr(PLAIN_RVA) as u32, 24),
        (game_addr(SCENE_RVA) as u32, 24),
        (game_addr(LAMPS[0].1) as u32, 48),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x1164_7AB5u32;
    for case in 0..24u32 {
        let (a, b) = [(0u32, 0u32), (1, 0), (0, 1), (0x100, 2)][case as usize % 4];
        let colours = [
            test_colours(&mut seed),
            test_colours(&mut seed),
            test_colours(&mut seed),
            test_colours(&mut seed),
        ];
        check_state(
            &mut t,
            &format!("case {case}: switches {a:#x} {b:#x}, colours {colours:x?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.0.len()]);
                game_write(OTHER_WAY_RVA, a);
                game_write(OTHER_WAY_B_RVA, b);
                game_write(PLAIN_RVA, colours[0]);
                game_write(SCENE_RVA, colours[1]);
                game_write(LAMPS[0].1, [colours[2], colours[3]]);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
