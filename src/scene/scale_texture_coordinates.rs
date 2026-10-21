//! `scene::scale_texture_coordinates` (0x44E2FB): the polygon's texture coordinates
//! from whole numbers to floats.
//!
//! In:  nothing
//! Out: eax = the last value worked on, ecx = the number of vertices, esi = the last
//!      entry; nothing changed for a polygon of no vertices; everything else preserved
//!
//! For each entry of the polygon, with the switch `dword_6240E0` its first pair of
//! coordinates (+8, +0xC) and with `dword_6240E4` its second (+0x10, +0x14): a value
//! that is not 0 becomes a float (`fild`, `fstp`) whose bits then lose 0x0C000000: the
//! number over 2^24. The two work variables `dword_5183C0` and `flt_5183C4` keep the
//! last value and its float.
//! Eight callers.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::Refused;
use crate::framework::{
    EAX, ECX, ESI, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::scene::draw_polygon::{SWITCH_A_RVA, SWITCH_B_RVA};
use crate::scene::polygon::{ENTRIES, ENTRIES_RVA, ENTRY_LEN, LIST_RVA, dword, listed, put};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::scale_texture_coordinates",
    rva(0x44E2FB),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const WORK_RVA: usize = rva(0x5183C0); // dword_5183C0, flt_5183C4

// ---- Rust version ----

// Converted: `rc2k_core::scene::texture_coordinates`
pub unsafe fn scale_texture_coordinates() -> Result<Left, Refused> {
    use rc2k_core::scene::texture_coordinates as coordinates;
    unsafe {
        if game_read::<i32>(COUNT_RVA) <= 0 {
            return Ok(Left::nothing());
        }
        let entries = listed()?;
        let switches: [u32; 2] = [game_read(SWITCH_A_RVA), game_read(SWITCH_B_RVA)];
        let mut eax = 0;
        for &entry in &entries {
            eax = (entry - game_addr(ENTRIES_RVA)) as u32;
            for (pair, switch) in switches.into_iter().enumerate() {
                if switch == 0 {
                    continue;
                }
                for at in [entry + 8 + pair * 8, entry + 0xC + pair * 8] {
                    eax = dword(at);
                    if let Some(float) = coordinates::float_of(eax) {
                        game_write(WORK_RVA, [eax, float]);
                    }
                    eax = coordinates::scaled(eax);
                    put(at, eax);
                }
            }
        }
        Ok(Left::nothing()
            .with(EAX, eax)
            .with(ECX, entries.len() as u32)
            .with(
                ESI,
                *entries.last().expect("the polygon has a vertex") as u32,
            ))
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![(game_addr(WORK_RVA) as u32, 8)];
    if unsafe { game_read::<i32>(COUNT_RVA) } > 0 {
        for entry in unsafe { listed() }.unwrap_or_default() {
            regions.push((entry as u32 + 8, 0x10));
        }
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { scale_texture_coordinates() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [
        (game_addr(ENTRIES_RVA) as u32, ENTRIES as usize * ENTRY_LEN),
        (game_addr(WORK_RVA) as u32, 8),
    ];
    let inputs = [
        (game_addr(LIST_RVA) as u32, 4 * 4),
        at(COUNT_RVA),
        at(SWITCH_A_RVA),
        at(SWITCH_B_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let values: [[u32; 4]; 4] = [
        [0, 1, 0x0100_0000, 0xFFFF_FFFF],
        [0x0080_0000, 0, 0x7FFF_FFFF, 0x8000_0000],
        [0x0123_4567, 0x00FF_FFFF, 0, 0],
        [3, 0x0200_0001, 0xFF00_0000, 0x0000_FFFF],
    ];
    for count in [0u32, 1, 3, 4, 0x8000_0000] {
        for (a, b) in [(0u32, 0u32), (1, 0), (0, 1), (5, 0x8000_0000)] {
            check_state(
                &mut t,
                &format!("{count:#x} vertices, switches {a:#x} {b:#x}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    restore(&regions, &vec![0xAA; saved.0.len()]);
                    for (n, value) in values.iter().enumerate() {
                        let number = [5u32, 2, 0x7F, 0][n];
                        game_write(LIST_RVA + n * 4, number);
                        let entry = game_addr(ENTRIES_RVA) + number as usize * ENTRY_LEN;
                        // Turned by the count, so that each entry is the last one once.
                        let mut value = *value;
                        value.rotate_left(count as usize % 4);
                        ((entry + 8) as *mut [u32; 4]).write_unaligned(value);
                    }
                    game_write(COUNT_RVA, count);
                    game_write(SWITCH_A_RVA, a);
                    game_write(SWITCH_B_RVA, b);
                },
                &regions,
                &rust,
            );
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
