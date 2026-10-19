//! `race::roll_credits` (0x415FC2): the credits rolling up (see `race/frame_text.rs`):
//! eax 0 starts them (clock, top, tune 2), else they are drawn from the top down by their
//! script, the top moving up a pixel each 0x12 ms; carry once they have rolled off and the
//! music has stopped.
//!
//! In:  eax = 0 to start
//! Out: every register preserved (`pusha` .. `popa`); CF as above, else ZF by eax
//!
//! Callers: `sub_4547AE` (over the race with `dword_71C3E0`), the credits' screen
//! (0x415F9F, 0x415FB7), `race::credits_race`. Shadow mode runs the original alone (it draws,
//! reads the clock and starts a tune). Self-test: the script drawn at tops above, on and
//! below the screen with the clock ahead (so it does not move), the music on and off, by
//! the drawing harness; the variables put back (skipped without the blitter's recording).

use crate::errors::Refused;
use crate::framework::{GameHook, RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::frame_text::{credits_tune, roll_credits};
use crate::screens::harness::{Left, check_draw, drawing_unavailable, push_dwords};
use crate::service::harness::{at, capture, restore};
use crate::text::draw_text::{Canvas, GameCanvas};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::roll_credits", rva(0x415FC2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() || crate::screens::blanked(HOOK.name()) {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { roll_credits(r.eax, &mut GameCanvas, &mut credits_tune, r) } {
        Ok(rolled) => {
            r.set_carry(rolled.carry);
            r.set_zero(rolled.zero);
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    let rolled = unsafe { roll_credits(r.eax, canvas, &mut |_| {}, r)? };
    Ok(Left::carry(rolled.carry))
}

fn state() -> Vec<u8> {
    let mut v =
        unsafe { std::slice::from_raw_parts(crate::strings::text_from_ids::text(), 0x40).to_vec() };
    push_dwords(&mut v, &[rva(0x52984C), rva(0x529850), rva(0x529854)]);
    v
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    let text = crate::strings::text_from_ids::text() as u32;
    let kept = [
        (text, 0x40),
        at(rva(0x52984C)),
        at(rva(0x529850)),
        at(rva(0x529854)),
        at(rva(0x506990)),
    ];
    let saved = capture(&kept);
    // One clock for both runs: the text buffer's bytes 40 .. 47 come out holding the top
    // and the clock (seen 2026-10-18; what copies them there is not yet read), so a clock
    // read in each run's set-up differed by the millisecond between them.
    let ahead = crate::system::tick_count::tick_count().wrapping_add(100_000);
    for (top, playing) in [(600i32, 1u32), (100, 1), (-2000, 1), (-20000, 0)] {
        check_draw(
            &mut t,
            &format!("top {top} music {playing}"),
            call_original,
            Regs {
                eax: 1,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x52984C), top as u32);
                game_write(rva(0x529850), ahead);
                game_write(rva(0x506990), playing);
            },
            &rust,
            &state,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
