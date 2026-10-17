//! `race::index_of_kind` (0x437720): the place of car kind eax in `dword_6051E0` (0x22
//! kinds, the first of equals), 0 for none.
//!
//! In:  eax = the kind
//! Out: eax = the place (0 for none), ecx = the place (0x22 for none); the rest
//!      preserved
//!
//! See `race/race_shell.rs`. Callers: `sub_449B79` four times (each `mov`s eax away; no
//! flags read). Shadow mode: the Rust version and the original, registers compared.
//! Self-test: the game's table, every kind in it and some not, and a table of the test's
//! with repeats, put back.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::race_shell::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::index_of_kind", rva(0x437720), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { index_of_kind(r) };
    Ok(Left::nothing().with(EAX, g.eax).with(ECX, g.ecx))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let table: Region = (
        game_addr(KIND_ORDER_RVA) as u32,
        4 * KIND_ORDER_LEN as usize,
    );
    let saved = capture(&[table]);
    let game: Vec<u32> = saved
        .chunks(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    let test: Vec<u8> = (0..KIND_ORDER_LEN)
        .flat_map(|n| ((n * 7) % 13).to_le_bytes())
        .collect();
    for (what, bytes, kinds) in [
        (
            "the game's",
            saved.clone(),
            game.iter()
                .copied()
                .chain([0xFFFF_FFFF, 0x99])
                .collect::<Vec<_>>(),
        ),
        ("repeats", test.clone(), (0..15u32).collect::<Vec<_>>()),
    ] {
        for kind in kinds {
            check_state(
                &mut t,
                &format!("{what} table, kind {kind:#x}"),
                call_original,
                Regs {
                    eax: kind,
                    ..Regs::SENTINEL
                },
                &|| restore(&[table], &bytes),
                &[],
                &rust,
            );
        }
    }
    restore(&[table], &saved);
    t.finish()
}
