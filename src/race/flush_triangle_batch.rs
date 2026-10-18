//! `race::flush_triangle_batch` (0x45FDCF): texture `dword_5D1D04`, the batch's render states, its triangles drawn by the device and the batch emptied (see `race/frame_flush.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! The device's draw (`sub_41ADF5`) and the log's writers are the game's (section 15).
//! Caller: `sub_45429B`. Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose calls go to stand-ins, against the Rust version with the same: the batch
//! empty, drawn, and failing (logged).

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::frame_flush::*;
use crate::race::level_init::note;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::flush_triangle_batch",
    rva(0x45FDCF),
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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { flush_batch(r, &PLAIN, false, &mut batch_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// What the draw's stand-in gives in eax.
static DRAWN: AtomicU32 = AtomicU32::new(0);

fn handler(place: u32, r: &mut Regs) {
    note(place, r);
    if place == 3 {
        r.eax = DRAWN.load(Ordering::SeqCst);
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(handler);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45FDCF,
            len: 0xAA,
            entry: &[0xA1, 0x04, 0x1D, 0x5D, 0x00, 0xE8, 0xA5, 0xE9],
            calls: &[0x05, 0x14, 0x23, 0x32, 0x41, 0x50, 0x77, 0x88, 0x8F],
            jumps_out: &[],
        },
        &|c| {
            [
                (0x44E77Eusize, 1u32),
                (0x44EB72, 2),
                (0x41ADF5, 3),
                (0x401427, 4),
                (0x401622, 5),
            ]
            .iter()
            .find(|(a, _)| rva(*a) == c)
            .map(|(_, p)| crate::install::stand_ins::stand_in(*p))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let b = &PLAIN;
    let regions: Vec<Region> = vec![
        at(rva(b.count as usize)),
        at(rva(b.word as usize)),
        at(rva(b.kept as usize)),
        at(rva(0x6240D4)),
    ];
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x6240C4)),
        (game_addr(rva(0x5D1AE8)) as u32, 8),
        at(rva(0x5DC970)),
    ]);
    let saved = capture(&kept);
    for (count, drawn) in [(0u32, 0u32), (12, 0), (5, 0x8876_0001)] {
        let line = compare_plain(
            &format!("count {count} drawn {drawn:#x}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(b.count as usize), count);
                game_write(rva(b.word as usize), 0x77u32);
                game_write(rva(0x6240C4), 1u32);
                game_write(rva(0x5DC970), 9u32);
                game_write(rva(0x5D1AE8), [3u32, 4]);
                DRAWN.store(drawn, Ordering::SeqCst);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                flush_batch(r, b, false, &mut |p, r| {
                    handler(p, r);
                    Ok(())
                })
                .map(|_| ())
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    t.finish()
}
