//! `screens::draw_rally_stage_info` (0x489AC8): a rally stage's information page (heading,
//! the stage panel's body, best and fastest; see `screens/stage_info.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758C6A). Refuses (before anything) no stage record or
//! times table. Compared in shadow mode (`dispatch_draw`). Self-test: the rally's stages 0 .. 2, nothing drawn (skipped
//! without the fonts or the records).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::stage_info::*;
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_rally_stage_info",
    rva(0x489AC8),
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

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    let _ = r;
    unsafe { draw_rally_stage_info(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    capture(&[
        (crate::screens::line_buffer() as u32, 0x100),
        g(0x7559AC, 0x48),
        g(0x755A18, 8),
        g(0x75ACD0, 8),
        (crate::strings::text_from_ids::text() as u32, 0x200),
    ])
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

/// The stage's times (as `draw_rally_stage_info` finds them).
fn times() -> u32 {
    let record = stage_record();
    let table: u32 = unsafe { game_read(rva(0x7556E0)) };
    if record == 0 || table == 0 {
        return 0;
    }
    table.wrapping_add(
        unsafe { ((record + 0x3C) as *const u32).read_unaligned() }.wrapping_mul(0x618),
    )
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    let regions: Vec<Region> = vec![at(rva(0x7587F0))];
    let saved = capture(&regions);
    for stage in [0u32, 1, 2] {
        unsafe { game_write(rva(0x7587F0), stage) };
        let record = stage_record();
        if times() == 0
            || unsafe { crate::records::trakpak_entry::record_entry(record as *const u8) }.is_err()
        {
            continue;
        }
        check_draw(
            &mut t,
            &format!("stage {stage}"),
            call_original,
            Regs {
                edi: times(),
                ..Regs::SENTINEL
            },
            &|| unsafe { game_write(rva(0x7587F0), stage) },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
