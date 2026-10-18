//! `race::draw_frame_rate` (0x4562B2): the frame rate ("fps: n", `flt_6214F0` rounded) at
//! (0x190, 4) in the font at 0x51DE90 when it is not 0 and `dword_5EB100` is set;
//! `dword_5C6604` 2 (see `race/frame_text.rs`).
//!
//! In:  nothing
//! Out: as `text::draw_text` leaves them, or all kept (a `call` follows)
//!
//! Caller: `sub_4547AE` (0x4547D2), each frame. Shadow mode: the drawing harness (the Rust
//! version recorded, the original drawn, both compared). Self-test: rates 0, 30, -60, 1000
//! with the switch on and off; the text buffer and the variables put back (skipped without
//! the blitter's recording or the font).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::frame_text::draw_frame_rate;
use crate::screens::font_is_loaded;
use crate::screens::harness::{Left, check_draw, dispatch_draw, drawing_unavailable, push_dwords};
use crate::service::harness::{at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_frame_rate", rva(0x4562B2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_frame_rate(canvas) }
}

fn state() -> Vec<u8> {
    let mut v =
        unsafe { std::slice::from_raw_parts(crate::strings::text_from_ids::text(), 0x20).to_vec() };
    push_dwords(&mut v, &[rva(0x5C6604), rva(0x62400C)]);
    v
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    if !font_is_loaded(0x51DE90) {
        return t.skip("the font is not loaded");
    }
    let text = crate::strings::text_from_ids::text() as u32;
    let kept = [
        (text, 0x20),
        at(rva(0x5C6604)),
        at(rva(0x62400C)),
        at(rva(0x6214F0)),
        at(rva(0x5EB100)),
    ];
    let saved = capture(&kept);
    for (rate, on) in [
        (0.0f32, 1u32),
        (30.2, 1),
        (-60.0, 1),
        (1000.0, 1),
        (30.0, 0),
    ] {
        check_draw(
            &mut t,
            &format!("rate {rate} switch {on}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x6214F0), rate.to_bits());
                game_write(rva(0x5EB100), on);
            },
            &rust,
            &state,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
