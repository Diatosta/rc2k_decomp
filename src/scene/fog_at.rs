//! `scene::fog_at` (0x44E260): the fog's alpha at a depth: eax (0 when below 0) >> 13,
//! held below `g_fogTableLength` (the last entry past it), its byte of `g_fogTable` in the
//! top byte, flipped (`xor 0xFF000000`).
//!
//! In:  eax the depth (whole)
//! Out: eax the alpha in its top byte; the rest kept
//!
//! Callers: the marks' drawers (16). The callers go on with a `mov`: no flags read.
//! Refuses a table length of 0 or past the table's 0x1000 bytes (the original reads past
//! it). Its first five bytes hold a short jump: private copy, manual patch. Self-test: the original and the Rust version, depths by lot and at the edges.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("scene::fog_at", rva(0x44E260), thunk, &ORIGINAL)
    .private_copy(0x26)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TABLE_RVA: usize = rva(0x63661C); // g_fogTable: 0x1000 bytes
const LENGTH_RVA: usize = rva(0x63761C); // g_fogTableLength

/// The Rust version (see the header).
pub unsafe fn fog_at(depth: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::scene::fog::fog_at`.
    Ok(rc2k_core::scene::fog::fog_at(
        depth,
        unsafe { game_read(LENGTH_RVA) },
        |n| unsafe { game_read(TABLE_RVA + n) },
    )?)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let eax = unsafe { fog_at(r.eax) }?;
    Ok(left_of(&Regs { eax, ..*r }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let vars = [
        at(LENGTH_RVA),
        (crate::framework::game_addr(TABLE_RVA) as u32, 0x1000),
    ];
    let saved = capture(&vars);
    for (len, depth) in [
        (0x100i32, 0u32),
        (0x100, 0x1FFF),
        (0x100, 0x2000),
        (0x100, 0x1F_FFFF),
        (0x100, 0x20_0000),
        (0x100, 0xFFFF_FFFF),
        (0x1000, 0x7FFF_FFFF),
        (1, 0x12_3456),
        (0x800, 0x0FFF_E000),
    ] {
        check_state(
            &mut t,
            &format!("length {len:#x}, depth {depth:#x}"),
            call_original,
            Regs {
                eax: depth,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&vars, &saved);
                game_write(LENGTH_RVA, len);
                for n in 0..0x1000usize {
                    game_write(TABLE_RVA + n, (n * 7 + 3) as u8);
                }
            },
            &[],
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
