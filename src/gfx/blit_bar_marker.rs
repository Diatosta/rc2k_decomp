//! `gfx::blit_bar_marker` (0x46B5FC): draws one sprite at the point of a bar that a percentage
//! reaches, and moves the bar's x there.
//!
//! In:  nothing in registers except ebx, which is passed on as the palette at 0 percent or
//!      less; the bar is described by the bar globals (see `blit_bar::Bar`)
//! Out: every register preserved; above 0 percent the bar's x (0x755224) is the marker's x
//! Above 0 percent, a width * percent of 0 is fatal error 0x80701010 in the original, and the
//! palette given to the blitter is the marker's x. These sprites do not use a palette.

use crate::errors::FatalError;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::gfx::blit_bar::{Bar, Blit, checked_list, on_surface};
use crate::gfx::blit_glyph::{record_calls, recording_works};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::blit_bar_marker", rva(0x46B5FC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `palette_at_start` is what the caller has in ebx; it only reaches the blitter at 0 percent
/// or less, where the marker stays at the bar's x.
pub unsafe fn blit_bar_marker(palette_at_start: u32, blit: Blit) -> Result<(), FatalError> {
    // Converted: `rc2k_core::gfx::bars::marker`.
    use crate::gfx::blit_bar::{Bar, GameBar};
    rc2k_core::gfx::bars::marker(&mut GameBar(blit), &Bar::read().core(), palette_at_start)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();

    if !shadow() {
        // The bar's x is only changed once the length is known to be valid, so on an error
        // the original still starts from the same globals.
        let before = Bar::read();
        if let Err(e) = unsafe { blit_bar_marker(r.ebx, &mut on_surface) } {
            before.write();
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }

    // Shadow: Rust moves the bar's x too, so it is put back before the original runs.
    let before = Bar::read();
    let mut mine = Vec::new();
    let predicted = unsafe { blit_bar_marker(r.ebx, &mut checked_list(&mut mine)) };
    let x_r = Bar::read().x;
    before.write();
    if let Err(e) = predicted {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        return;
    }

    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });
    let x_o = Bar::read().x;

    let ok = o.preserved_except(r, 0) && x_o == x_r && (!recording_works() || calls == mine);
    if !ok {
        HOOK.mismatch(|| {
            format!("{before:?}: original calls={calls:?} x={x_o}; rust calls={mine:?} x={x_r}")
        });
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    // The bar's x is only changed once the length is known to be valid, so on an error
    // the original still starts from the same globals.
    let before = Bar::read();
    if let Err(e) = unsafe { blit_bar_marker(r.ebx, &mut on_surface) } {
        before.write();
        return Err(e.into());
    }
    Ok(())
}

// ---- self-test: the blitter's hook swallows the calls, so nothing is drawn ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off, so the original would really draw");
    }
    let saved = Bar::read();

    // Never a positive percent with a product of 0: that is the fatal error.
    for (width, percent) in [
        (120, 0),
        (120, -5),
        (120, 1),
        (120, 50),
        (120, 100),
        (120, 150),
        (7, 33),
        (1, 1),
        (-120, 25),
    ] {
        let bar = Bar {
            x: 200,
            y: 150,
            width,
            fill_glyph: 0x42,
            end_glyph: 0x43,
            percent,
        };
        bar.write();
        let before = Regs::SENTINEL;
        let mut ro = before;
        let calls_o = record_calls(true, || unsafe { call_original(&mut ro) });
        let after_o = Bar::read();

        bar.write();
        let mut calls_r = Vec::new();
        let mine = unsafe {
            blit_bar_marker(before.ebx, &mut |call| {
                calls_r.push(call);
                Ok(())
            })
        };
        let after_r = Bar::read();

        let preserved = ro.preserved_except(&before, 0);
        t.check(
            mine.is_ok() && calls_o == calls_r && after_o == after_r && preserved,
            || {
                format!(
                    "width={width} percent={percent}: original calls={calls_o:?} {after_o:?} preserved={preserved}; rust {mine:?} calls={calls_r:?} {after_r:?}"
                )
            },
        );
    }

    saved.write();
    t.finish()
}
