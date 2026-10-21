//! `hud::set_batch_texture` (0x45A855): the device's texture set to the batch's
//! (`g_kindFirstSlots`[`dword_6D14FE`], `scene::set_texture`).
//!
//! In:  nothing
//! Out: all kept (`pusha` .. `popa`)
//!
//! Caller: `hud::add_polygon`. The texture call through `part` (1 with eax the texture),
//! in the game its Rust version. Refuses a texture past `g_kindFirstSlots` (0x400 dwords; the HUD asks for 0xC0 .. 0xDC), before
//! anything. Shadow mode runs the original alone (it tells the device). Self-test: a copy
//! whose call goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::hud::batch::SHOWN_TEXTURE_RVA;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::set_batch_texture", rva(0x45A855), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SLOTS_RVA: usize = rva(0x5D1AE8); // g_kindFirstSlots

/// A call by place: 1 `set_texture`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::batch::texture_slot`
pub unsafe fn set_batch_texture(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let n = rc2k_core::hud::batch::texture_slot(unsafe { game_read(SHOWN_TEXTURE_RVA) })?;
    let mut r = Regs {
        eax: unsafe { game_read(SLOTS_RVA + 4 * n) },
        ..*entry
    };
    part(1, &mut r)?;
    Ok(*entry)
}

/// In the game: `set_texture`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::scene::set_texture::set_texture_in_game(r) };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { set_batch_texture(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov eax, dword_6D14FE
    let entry: &[u8] = &[0x60, 0xA1, 0xFE, 0x14, 0x6D, 0x00];
    let copy = match copy(0x45A855, 0x14, entry, &[0xD], &[(0x44E77E, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![at(SHOWN_TEXTURE_RVA)];
    let saved = capture(&regions);
    for n in [0u32, 1, 0x40, 0x86] {
        run_both(
            &mut t,
            &format!("texture {n:#x}"),
            &copy,
            &regions,
            &|| unsafe { game_write(SHOWN_TEXTURE_RVA, n) },
            &|r| unsafe { set_batch_texture(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
