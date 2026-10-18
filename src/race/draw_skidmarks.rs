//! `race::draw_skidmarks` (0x458C03): the skidmarks shown in the drawn view, a quad for
//! each link with a link before (see `race/skidmarks.rs`; the x87 runs the original's
//! instructions, the turn by `scene::rotate_point`'s Rust version), each faded
//! (`scene::fog_polygon`), cut by depth and to the screen and drawn in black in mode 3
//! without the depth test (`scene::depth_test_off`, `scene::draw_polygon_in_game`,
//! `scene::depth_test_on`).
//!
//! In:  nothing
//! Out: edx 0x400, eax, ecx, esi as it leaves them (a `retn` follows)
//!
//! Caller: `race::draw_skidmarks_if_on`. Refuses what the scene's functions refuse
//! (mid-way; not met with four vertices). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose depth-test and drawing calls go to stand-ins (the fade and the
//! clips real), against the Rust version on the same; chains of links of the test's in
//! the views and not; all put back (skipped without a race).

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::level_init::{note, take_called};
use crate::race::skidmarks::*;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_skidmarks", rva(0x458C03), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { draw_skidmarks(r, &mut skid_part) } {
        Ok(after) => {
            let esp = r.esp;
            *r = after;
            r.esp = esp;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// What a skidmark test watches and puts back.
pub fn skid_regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x975510, 0x4AF0),
        g(0x623E40, 0x80),
        g(0x6240A4, 0x94),
        at(rva(0x63DAB4)),
        at(rva(0x643AE4)),
        at(rva(0x62400C)),
    ]
}

/// The skidmark inputs a test sets.
pub fn skid_inputs() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x643AE8, 0x2000),
        g(0x645B18, 0x6000),
        at(rva(0x643AD4)),
        at(rva(0x62389C)),
        at(rva(0x63DB9C)),
        at(rva(0x6416B8)),
    ]
}

/// A test's skidmarks: a chain of `n` links (each one's link before the last, views by
/// `views`), points along the road ahead of the view.
pub fn set_skids(n: u32, views: u16, mode: u32) {
    unsafe {
        let links = game_addr(rva(0x643AE8)) as u32;
        let points = game_addr(rva(0x645B18)) as u32;
        for k in 0..0x400u32 {
            ((links + 8 * k) as *mut i16).write_unaligned(-1);
        }
        let view = game_addr(rva(0x606298)) as u32;
        let (vx, vy, vz) = (
            (view as *const u32).read_unaligned(),
            ((view + 4) as *const u32).read_unaligned(),
            ((view + 8) as *const u32).read_unaligned(),
        );
        for k in 0..n {
            ((links + 8 * k) as *mut i16).write_unaligned(if k == 0 { -1 } else { k as i16 - 1 });
            ((links + 8 * k + 6) as *mut u16).write_unaligned(views ^ (k as u16 & 1));
            for s in 0..2u32 {
                let p = points + 0xC * (2 * k + s);
                (p as *mut u32).write_unaligned(vx.wrapping_add(0x8000 * s).wrapping_sub(0x4000));
                ((p + 4) as *mut u32).write_unaligned(vy.wrapping_sub(0x1_0000));
                ((p + 8) as *mut u32)
                    .write_unaligned(vz.wrapping_add(0x2_0000 * k).wrapping_sub(0x4_0000));
            }
        }
        game_write(rva(0x643AD4), 64.0f32.to_bits());
        game_write(rva(0x62389C), mode);
        game_write(rva(0x63DB9C), 1u32);
        game_write(rva(0x6416B8), 0u32);
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(rva(0x621480)) } == 0 {
        return t.skip("no race loaded");
    }
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x458C03,
            len: 0x513,
            entry: &[0xC7, 0x05, 0xE4, 0x3A, 0x64, 0x00, 0xFF, 0xFF],
            calls: &[
                0x1A1, 0x257, 0x30C, 0x3C2, 0x48E, 0x493, 0x4A1, 0x4B9, 0x4DC, 0x4F5,
            ],
            jumps_out: &[],
        },
        &|c| {
            SKID_CALLEES
                .iter()
                .find(|&&(_, k)| rva(k) == c)
                .map(|&(p, _)| crate::install::stand_ins::stand_in(p))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let regions = skid_regions();
    let mut kept = regions.clone();
    kept.extend(skid_inputs());
    let saved = capture(&kept);
    for (n, views, mode) in [
        (0u32, 0xFFu16, 0u32),
        (5, 0x02, 1),
        (8, 0x01, 1),
        (6, 0x00, 2),
    ] {
        let prepare = || {
            restore(&kept, &saved);
            set_skids(n, views, mode);
            take_called();
        };
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (regs(&ro), take_called(), capture(&regions));
        prepare();
        crate::race::skidmarks::SKIDS_CLIPPED.store(false, std::sync::atomic::Ordering::SeqCst);
        let done = unsafe {
            draw_skidmarks(&Regs::SENTINEL, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (
            done.as_ref().map(regs).unwrap_or_default(),
            take_called(),
            capture(&regions),
        );
        // A quad cut by the screen clip: the calls compared by place (see SKIDS_CLIPPED).
        let same =
            if crate::race::skidmarks::SKIDS_CLIPPED.load(std::sync::atomic::Ordering::SeqCst) {
                theirs.0 == mine.0
                    && theirs.2 == mine.2
                    && theirs.1.iter().map(|c| c.0).eq(mine.1.iter().map(|c| c.0))
            } else {
                theirs == mine
            };
        t.check(same, || {
            let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!("links {n} views {views:#x} mode {mode}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}", theirs.0, theirs.1, mine.0, mine.1)
        });
    }
    restore(&kept, &saved);
    t.finish()
}
