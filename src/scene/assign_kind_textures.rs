//! `scene::assign_kind_textures` (0x4256D8): every texture slot gets the texture of
//! the kind it is the first slot of.
//!
//! In:  nothing
//! Out: eax = kind 0's first slot, ebx = kind 0's first texture, ecx = 0xFFFFFFFF,
//!      edi = the end of the slots' table
//!
//! The 0x400 slots of `dword_5DA968` = -1; then for the kinds from 0x3FF down to 0:
//! the kind's first slot (`dword_5D1AE8`), unless it is -1, gets the kind's first
//! texture (`dword_5D0AE8`). So of two kinds with one slot the lower one's stays.
//! Six callers.
//! Refuses a first slot past the table.
//! Self-test: 8 tables by lot in the game's places, put back afterwards.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, ECX, EDI, RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::scene::textures::{
    KIND_SLOT_RVA, KIND_TEXTURE_RVA, KINDS, SLOTS, WANTED_RVA, entry, slice, table, test_table,
    write_table,
};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::assign_kind_textures",
    rva(0x4256D8),
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

// ---- Rust version ----

pub unsafe fn assign_kind_textures() -> Result<Left, BadInput> {
    // Converted: `rc2k_core::scene::textures::assign_kind_textures`.
    unsafe {
        rc2k_core::scene::textures::assign_kind_textures(
            slice(KIND_SLOT_RVA, KINDS),
            slice(KIND_TEXTURE_RVA, KINDS),
            slice(WANTED_RVA, SLOTS),
        )?;
        Ok(Left::nothing()
            .with(EAX, entry(KIND_SLOT_RVA, 0))
            .with(EBX, entry(KIND_TEXTURE_RVA, 0))
            .with(ECX, u32::MAX)
            .with(EDI, game_addr(WANTED_RVA) as u32 + SLOTS * 4))
    }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { assign_kind_textures() }?)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[table(WANTED_RVA, SLOTS)], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [table(WANTED_RVA, SLOTS)];
    let inputs = [table(KIND_SLOT_RVA, KINDS), table(KIND_TEXTURE_RVA, KINDS)];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x7E87_0001u32;
    for case in 0..8u32 {
        // Few slots, so that kinds share them.
        let slots = test_table(
            &mut seed,
            KINDS,
            1 + case % 4,
            [SLOTS, 8, 0x40][case as usize % 3],
        );
        let textures = test_table(&mut seed, KINDS, 1, 0x1_0000);
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.0.len()]);
                write_table(KIND_SLOT_RVA, &slots);
                write_table(KIND_TEXTURE_RVA, &textures);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
