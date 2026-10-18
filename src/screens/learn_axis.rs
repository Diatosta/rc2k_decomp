//! `screens::learn_axis` (0x46B131): the controller page's learning of a control's axis,
//! direction or button for up to 3 s (see `screens/controller.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In four controller pages' draw lists (0x758D96 ..). Refuses (before anything) no current
//! player record. Shadow mode runs the original alone (the clock, the key and the draw).
//! Self-test: a copy whose clock, prompt, axis, read and key calls go to stand-ins giving
//! the case's values (the classifying real), against the Rust version with the same: time
//! out, the axis learnt or not, the controller's button, space; all put back (skipped
//! without a player).

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::screens::controller::{learn_axis, learn_part};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::learn_axis", rva(0x46B131), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::screens::blanked("screens::learn_axis") {
        return;
    }
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { learn_axis(r, &mut learn_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

static TIME: AtomicU32 = AtomicU32::new(0);
static MOVED: AtomicU32 = AtomicU32::new(0);
static STILL: AtomicBool = AtomicBool::new(false);
static BUTTONS: AtomicU32 = AtomicU32::new(0);
static KEY: AtomicU32 = AtomicU32::new(0);

fn handler(place: u32, r: &mut Regs) {
    note(place, r);
    match place {
        1 => r.eax = TIME.load(Ordering::SeqCst),
        3 => {
            r.eax = MOVED.load(Ordering::SeqCst);
            r.set_carry(STILL.load(Ordering::SeqCst));
        }
        4 => (r.ecx, r.edx, r.eax, r.ebx) = (5, 0, 0, BUTTONS.load(Ordering::SeqCst)),
        5 => r.eax = (r.eax & !0xFF) | KEY.load(Ordering::SeqCst),
        _ => {}
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let player = crate::menu::player::player_at(0);
    if player == 0 {
        return t.skip("no player record");
    }
    crate::install::stand_ins::set_handler(handler);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x46B131,
            len: 0x149,
            entry: &[0x60, 0x83, 0x3D, 0xF4, 0x51, 0x75, 0x00, 0x01],
            calls: &[0x18, 0x73, 0x98, 0xCE, 0xF6, 0xFB, 0x100],
            jumps_out: &[],
        },
        &|c| {
            [
                (0x411024usize, 1u32),
                (0x40234A, 2),
                (0x40939F, 3),
                (0x46B4AE, 4),
                (0x407F95, 5),
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
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let regions: Vec<Region> = vec![
        g(0x7551E4, 0x1C),
        at(rva(0x7559EC)),
        at(rva(0x4F6F08)),
        (player + 0x34, 0x40),
    ];
    let mut kept = regions.clone();
    kept.extend([at(rva(0x75598C)), at(rva(0x7559A4))]);
    let saved = capture(&kept);
    // (time since the start, the player's kind, control, axis moved (none: still), buttons, key)
    for (time, kind, control, moved, buttons, key) in [
        (5000u32, 3u32, 1u32, None, 0u32, 0u32),
        (100, 3, 1, Some(0x12u32), 0, 0),
        (100, 3, 2, None, 0, 0x39),
        (100, 3, 2, Some(3), 0, 0),
        (100, 1, 1, None, 4, 0),
        (100, 1, 3, None, 0, 0x39),
        (100, 1, 4, None, 0, 0),
    ] {
        let line = compare_plain(
            &format!(
                "time {time} kind {kind} control {control} moved {moved:?} buttons {buttons:#x} key {key:#x}"
            ),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x75598C), 0u32);
                game_write(rva(0x7559A4), 0u32);
                game_write(rva(0x7551F4), 1u32);
                game_write(rva(0x7551F8), 1000u32);
                game_write(rva(0x7551FC), control);
                ((player + 0x34) as *mut u32).write_unaligned(kind);
                TIME.store(1000 + time, Ordering::SeqCst);
                STILL.store(moved.is_none(), Ordering::SeqCst);
                MOVED.store(moved.unwrap_or(0), Ordering::SeqCst);
                BUTTONS.store(buttons, Ordering::SeqCst);
                KEY.store(key, Ordering::SeqCst);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                learn_axis(r, &mut |p, r| {
                    handler(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    t.finish()
}
