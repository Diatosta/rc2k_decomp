//! `marks::measure_lines` (0x4D0EB4): the lines between the blocks' two objects' faces
//! summed, by kind, into the blocks, and made shares.
//!
//! In:  nothing
//! Out: as the last block leaves them (see `marks/lines.rs`)
//!
//! Callers: `sub_4D0DB5` (step 14) and `sub_4D08DD`. Refuses more than 8 blocks, an object
//! past 0xFF, faces past `g_faces`. Its `finit` (after the first six shares) is done by the
//! Rust version as well: in shadow mode each run starts from the control word the call
//! came with. Self-test: the original and the Rust version on scenes of the test's (every
//! block or the marked ones, with and without the shares, 0 .. 8 blocks), the refusal; put
//! back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::lines::*;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::measure_lines", rva(0x4D0EB4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The control word each run starts from.
static ENTRY_WORD: AtomicU16 = AtomicU16::new(0x37F);

unsafe extern "C" fn original_from_entry_word(r: *mut Regs) {
    x87::set_control_word(ENTRY_WORD.load(Ordering::SeqCst));
    unsafe { call_original(r) };
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    x87::set_control_word(ENTRY_WORD.load(Ordering::SeqCst));
    Ok(left_of(&unsafe { measure_lines(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    ENTRY_WORD.store(x87::control_word(), Ordering::SeqCst);
    dispatch_state(&HOOK, r, original_from_entry_word, &line_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let word = x87::control_word();
    ENTRY_WORD.store(word, Ordering::SeqCst);
    let scene = TestScene::new();
    let globals = [
        at(CURRENT_RVA),
        at(ANY_RVA),
        (crate::framework::game_addr(DIFFERENCE_RVA) as u32, 12),
    ];
    let saved = capture(&globals);
    let regions = line_regions();
    for every in [0u32, 1] {
        for shares in [0u32, 1] {
            for blocks in [0u32, 1, 3, 8] {
                for seed in [1u32, 2] {
                    check_state(
                        &mut t,
                        &format!("every {every}, shares {shares}, {blocks} blocks, seed {seed}"),
                        original_from_entry_word,
                        Regs::SENTINEL,
                        &|| unsafe {
                            scene.fill(seed * 7 + blocks, blocks);
                            game_write(EVERY_RVA, every);
                            game_write(SHARES_RVA, shares);
                        },
                        &regions,
                        &rust,
                    );
                }
            }
        }
    }
    scene.fill(5, 9);
    t.check(unsafe { measure_lines(&Regs::SENTINEL) }.is_err(), || {
        "9 blocks: the Rust version went on".into()
    });
    x87::set_control_word(word);
    restore(&globals, &saved);
    drop(scene);
    t.finish()
}
