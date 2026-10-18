//! `scene::texture_picture_folder` (0x425A59): the folder and name pattern of a
//! texture's picture, by its number (`dword_60AF04`): below 900, 902 .. 999 the first
//! pair, 900 and 901 the second, 1000 .. 1999, 2000 .. 2999 and 3000 on their own.
//!
//! In:  nothing
//! Out: ebx the folder, ebp the pattern (strings in the game's data); everything else
//!      preserved
//! One caller, 0x425985 (`scene::load_texture_picture`, whose Rust version picks them
//! itself). Shadow mode: both, compared.
//! Self-test: the original against the Rust version for numbers around the bounds (the
//! game's variable, put back).

use crate::errors::Refused;
use crate::framework::{
    EBP, EBX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::texture_picture_folder",
    rva(0x425A59),
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

/// dword_60AF04: the texture's number.
const NUMBER_RVA: usize = rva(0x60AF04);
/// (from this number on, folder, pattern), in the original's order.
const PAIRS: [(u32, usize, usize); 6] = [
    (0, 0x60AF0C, 0x60AF8F),
    (900, 0x60AF1F, 0x60AFA2),
    (902, 0x60AF0C, 0x60AF8F),
    (1000, 0x60AF32, 0x60AFB5),
    (2000, 0x60AF5F, 0x60AFE2),
    (3000, 0x60AF77, 0x60AFFA),
];

// ---- Rust version ----

/// (folder, pattern) as IDA addresses.
pub fn texture_picture_folder(number: u32) -> (usize, usize) {
    let (_, folder, pattern) = PAIRS.iter().rev().find(|p| number >= p.0).copied().unwrap();
    (folder, pattern)
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    let _ = r;
    let (folder, pattern) = texture_picture_folder(unsafe { game_read(NUMBER_RVA) });
    Ok(Left::nothing()
        .with(EBX, game_addr(rva(folder)) as u32)
        .with(EBP, game_addr(rva(pattern)) as u32))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = [at(NUMBER_RVA)];
    let saved = capture(&all);
    for number in [
        0u32,
        899,
        900,
        901,
        902,
        999,
        1000,
        1999,
        2000,
        2999,
        3000,
        0xFFFF_FFFF,
    ] {
        check_state(
            &mut t,
            &format!("number {number}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(NUMBER_RVA, number) },
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
