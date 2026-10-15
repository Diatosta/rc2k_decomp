//! `audio::volume_index` (0x40B8D1): a DirectSound volume back to an index of the volume
//! table.
//!
//! In:  eax = the volume (signed, hundredths of a decibel)
//! Out: eax = the first index 0 .. 0x3F whose `g_volumeTable` entry is at least it, else
//!      0x40; everything else preserved
//!
//! Its loop goes back to 0x40B8D4, inside the patched bytes: a private copy (the first
//! build without it ended the game in its self-test, 2026-10-15). One caller,
//! `audio::make_primary`. See `audio/primary.rs`. Compared in shadow mode.
//! Self-test: the original on every table entry, one below and above each, and extremes.

use crate::audio::primary::volume_index;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("audio::volume_index", rva(0x40B8D1), thunk, &ORIGINAL)
    .private_copy(0x16)
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = volume_index(r.eax as i32);
    if !shadow() {
        r.eax = mine;
        return;
    }
    let volume = r.eax;
    unsafe { call_original(r) };
    if r.eax != mine {
        HOOK.mismatch(|| format!("volume {volume:#x}: original {:#x}, rust {mine:#x}", r.eax));
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut values = vec![i32::MIN, i32::MAX, 0, 1, -1, -10_001];
    for n in 0..0x41 {
        let v: i32 = unsafe { game_read(crate::audio::voices::VOLUMES_RVA + n * 4) };
        values.extend([v.wrapping_sub(1), v, v.wrapping_add(1)]);
    }
    for v in values {
        let before = Regs {
            eax: v as u32,
            ..Regs::SENTINEL
        };
        let mut r = before;
        unsafe { call_original(&mut r) };
        let mine = volume_index(v);
        t.check(r.eax == mine && r.preserved_except(&before, EAX), || {
            format!("volume {v}: original {:#x}, rust {mine:#x}", r.eax)
        });
    }
    t.finish()
}
