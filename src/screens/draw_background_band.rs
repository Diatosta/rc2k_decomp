//! `screens::draw_background_band` (0x4698C4): draws a band of the background again.
//!
//! In:  ebx (the sprite's palette)
//! Out: every register preserved
//! Only while `dword_7554E4` is 1: sprite 0xA0 of the sprite table (the background
//! picture) at (0, 0) with the clip rectangle's top and bottom at 0xCB and 0x1C6 for
//! that one blit, and put back after it.
//! In 15 lists of screens (the setup and saved game lists, the service area set the
//! switch). What the band covers has not been seen: the name is a reading.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::gfx::blit_marker_pair::sprite_table;
use crate::gfx::surface::{CLIP_BOTTOM_RVA, CLIP_TOP_RVA};
use crate::screens::harness::{Left, check_draw, dispatch_draw, drawing_unavailable};
use crate::service::harness::{at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_background_band",
    rva(0x4698C4),
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

const SWITCH_RVA: usize = rva(0x7554E4); // dword_7554E4

// ---- Rust version ----

pub unsafe fn draw_background_band(palette: u32, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::controller_page::background_band`.
    use rc2k_core::screens::controller_page::{Font, Work, background_band};
    background_band(
        &mut crate::screens::game_screen::GameScreen::<Font, Work>::new(canvas),
        unsafe { game_read::<u32>(SWITCH_RVA) } == 1,
        palette,
    )
}

// ---- hook ----

fn state() -> Vec<u8> {
    capture(&[at(CLIP_TOP_RVA), at(CLIP_BOTTOM_RVA)])
}

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_background_band(r.ebx, canvas) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    if sprite_table() == 0 {
        return t.skip("the sprites are not built");
    }
    let switch = [at(SWITCH_RVA)];
    let saved = capture(&switch);
    for value in [0u32, 1, 2, 0xFFFF_FFFF] {
        check_draw(
            &mut t,
            &format!("switch {value:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(SWITCH_RVA, value) },
            &rust,
            &state,
        );
    }
    restore(&switch, &saved);
    t.finish()
}
