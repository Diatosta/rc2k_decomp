//! `scene::map_texture_places` (0x42570D): for each texture's place, the slot that
//! shows it.
//!
//! In:  nothing
//! Out: eax = what was last looked at, ecx = 0xFFFFFFFF, edi = the end of the table
//!
//! The 0xFA0 dwords of `dword_5D3AE8` = -1; then for the slots from 0x3FF down to 0:
//! the texture the slot wants (`dword_5DA968`), if under 0x400, has a place
//! (`dword_5D8968`), and that place, if under 0xFA0, gets the slot's number. Of two
//! slots with one place the lower stays.
//! Six callers.
//! Self-test: 8 tables by lot in the game's places, put back afterwards.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDI, RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::scene::textures::{
    PLACE_SLOT_RVA, PLACES, SLOTS, TEXTURE_PLACE_RVA, WANTED_RVA, slice, table, test_table,
    write_table,
};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::map_texture_places", rva(0x42570D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn map_texture_places() -> Left {
    // Converted: `rc2k_core::scene::textures::map_places`.
    unsafe {
        let eax = rc2k_core::scene::textures::map_places(
            slice(WANTED_RVA, SLOTS),
            slice(TEXTURE_PLACE_RVA, SLOTS),
            slice(PLACE_SLOT_RVA, PLACES),
        );
        Left::nothing()
            .with(EAX, eax)
            .with(ECX, u32::MAX)
            .with(EDI, game_addr(PLACE_SLOT_RVA) as u32 + PLACES * 4)
    }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { map_texture_places() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[table(PLACE_SLOT_RVA, PLACES)],
        &rust,
    );
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [table(PLACE_SLOT_RVA, PLACES)];
    let inputs = [table(WANTED_RVA, SLOTS), table(TEXTURE_PLACE_RVA, 0x400)];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x7E87_0002u32;
    for case in 0..8u32 {
        let wanted = test_table(
            &mut seed,
            SLOTS,
            1 + case % 3,
            [0x400, 0x500, 0x20][case as usize % 3],
        );
        let places = test_table(
            &mut seed,
            0x400,
            1 + case % 2,
            [PLACES, PLACES + 0x100, 0x10][case as usize % 3],
        );
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.0.len()]);
                write_table(WANTED_RVA, &wanted);
                write_table(TEXTURE_PLACE_RVA, &places);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
