//! `screens::draw_service_stage_info` (0x480064): the stage panel opened from the service
//! area. It is `draw_stage_info`'s panel with two differences at the top: the stage heading
//! is drawn only in mode 2 with `dword_755968` at 0, and a second heading text (string 0x482)
//! is built in its own buffer without being drawn here.
//!
//! In:  nothing in registers; it works from the section and index of the event's stage
//! Out: carry as `draw_stage_records` leaves it; every register preserved
//! Reached through the draw list at 0x758D86 of the screen loop `sub_47FD37`, which the menu
//! action `sub_47037E` starts when the event type `dword_755954` is not 3. That action is the
//! last of 22 items (all at 747, 551) of the service area's menu at 0x74D24C.
//!
//! Screen: confirmed by the owner on 2026-10-10: the stage details that can be opened from the
//! service area of a championship, the screen after the first stage details where the car is
//! set up and repaired before the stage. Named `draw_stage_info_alt` until then.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::screens::draw_stage_heading::draw_stage_heading;
use crate::screens::draw_stage_info::{
    MODE_RVA, TITLE_X, TITLE_Y, event_stage, event_stage_unusable, event_title,
};
use crate::screens::stage_panel::{
    BEST_TIMES_PTR_RVA, TITLE_FONT, body_unavailable, build_second_heading, check_panel,
    dispatch_panel, draw_records_from, draw_stage_body, label,
};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_service_stage_info",
    rva(0x480064),
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

const HEADING_SWITCH_RVA: usize = rva(0x755968); // dword_755968: 0 lets mode 2 draw the heading

// ---- Rust version ----

/// Returns the carry flag of `draw_stage_records`.
pub unsafe fn draw_service_stage_info(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    unsafe {
        label(canvas, event_title(), TITLE_FONT, TITLE_X, TITLE_Y)?;
        let (record, section) = event_stage();
        if game_read::<u32>(MODE_RVA) == 2 && game_read::<u32>(HEADING_SWITCH_RVA) == 0 {
            draw_stage_heading(record as *const u8, canvas)?;
        }
        build_second_heading(record)?;
        // ebx, the markers' palette, still holds the section here.
        draw_stage_body(record, section, false, canvas)?;
        draw_records_from(game_read(BEST_TIMES_PTR_RVA), record, canvas)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_panel(&HOOK, r, call_original, &|canvas| unsafe {
        draw_service_stage_info(canvas)
    });
}

// ---- self-test: one case, the event's current stage (see draw_stage_info) ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = body_unavailable().or_else(|| unsafe { event_stage_unusable() }) {
        return t.skip(why);
    }
    check_panel(&mut t, call_original, &|canvas| unsafe {
        draw_service_stage_info(canvas)
    });
    t.finish()
}
