//! `scene::look_up_texture` (0x44E668): `g_polygonTexture` for the texture word
//! `dword_6240C4` (stream A, part R3): -1 none; with the top bit a place
//! (`dword_5D3AE8[]`, below 0xFA0); else a kind's first slot (`g_kindFirstSlots[]`, below
//! 0x400); a slot not below `g_textureSlotCount` (or none loaded) none (0xFFFFFC43).
//!
//! In:  nothing (eax is not read: its caller 0x45FF6C loads one it does not use)
//! Out: eax as it leaves it (the word, or the slot); everything else kept
//!
//! Callers: 0x44CCB8, `race::flush_texture_batch`. Shadow mode: by state. Self-test: words
//! of each kind and the edges, put back.

use crate::errors::Refused;
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::look_up_texture", rva(0x44E668), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const WORD_RVA: usize = rva(0x6240C4); // dword_6240C4
const PLACES_RVA: usize = rva(0x5D3AE8); // dword_5D3AE8
const FIRST_SLOTS_RVA: usize = rva(0x5D1AE8); // g_kindFirstSlots
const SLOT_COUNT_RVA: usize = rva(0x5DC970); // g_textureSlotCount
const TEXTURE_RVA: usize = rva(0x6240D4); // g_polygonTexture

/// The Rust version: what it leaves in eax.
pub unsafe fn look_up_texture() -> u32 {
    // Converted: `rc2k_core::scene::textures::look_up`.
    unsafe {
        let found = rc2k_core::scene::textures::look_up(
            game_read(WORD_RVA),
            |p| game_read(PLACES_RVA + 4 * p),
            |k| game_read(FIRST_SLOTS_RVA + 4 * k),
            || game_read(SLOT_COUNT_RVA),
        );
        game_write(TEXTURE_RVA, found.texture);
        found.eax
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { look_up_texture() }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(TEXTURE_RVA)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions: [Region; 1] = [at(TEXTURE_RVA)];
    let kept: Vec<Region> = vec![
        at(TEXTURE_RVA),
        at(WORD_RVA),
        at(SLOT_COUNT_RVA),
        (game_addr(PLACES_RVA) as u32, 0x10),
        (game_addr(FIRST_SLOTS_RVA) as u32, 0x10),
    ];
    let saved = capture(&kept);
    for (word, count) in [
        (u32::MAX, 10u32),
        (0, 10),
        (2, 10),
        (3, 2),
        (0x400, 10),
        (0x8000_0001, 10),
        (0x8000_0FA0, 10),
        (0x8000_0003, 0),
        (1, 0),
    ] {
        check_state(
            &mut t,
            &format!("word {word:#x} slots {count}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(WORD_RVA, word);
                game_write(SLOT_COUNT_RVA, count);
                game_write(PLACES_RVA, [5u32, 1, 9, 3]);
                game_write(FIRST_SLOTS_RVA, [0u32, 7, 4, 12]);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
