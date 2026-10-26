//! `scene::init_texture_levels` (0x424001): the stage's texture slots, their levels and
//! tags (`LOADLEVEL_INIT : init ed textures`).
//!
//! In:  nothing
//! Out: see `scene::texture_levels::init_texture_levels`
//!
//! See `scene/texture_levels.rs`. One caller, `sub_44426F` (loadlevel_init). Shadow mode
//! runs the original alone (it logs and takes the frame's input). Self-test: a redirected
//! copy whose reset, log and input calls note themselves (its runs on their hook),
//! against the Rust version noting the same: 0, 1, 3 and 9 cars, `dword_63DB94` 1 and
//! not, `dword_621514` 0 and not, prefs entries of the test's.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::scene::texture_levels::*;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::init_texture_levels",
    rva(0x424001),
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
    match unsafe { init_texture_levels(r, &mut game_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { init_texture_levels(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

const CALLS: &[usize] = &[
    0x26, 0x32, 0x3e, 0x5b, 0x79, 0xa0, 0xc0, 0xeb, 0x109, 0x134, 0x152, 0x17d, 0x19b, 0x1c6,
    0x1e4, 0x20f, 0x22d, 0x258, 0x276, 0x29d, 0x2bb, 0xeef, 0xf0d, 0xf2b, 0xf49, 0xf67, 0xf85,
    0xfa3, 0xfc1, 0xfdf, 0xffd, 0x101b, 0x1039, 0x1057, 0x1075, 0x1093, 0x1121, 0x1148, 0x1166,
    0x1184,
];

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x424001,
        0x118A,
        &[0x83, 0x3D, 0x90, 0xDB, 0x63, 0x00, 0x01],
        CALLS,
        &[(0x42247B, 0), (0x401427, 1), (0x401476, 2), (0x420115, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let prefs: Region = (crate::framework::game_addr(rva(0x60A57C)) as u32, 0x220);
    let mut watched = tables();
    watched.extend([at(OTHER_LEVELS_RVA), at(KIND_SLOTS_RVA), at(CARS_RVA)]);
    let mut all = watched.clone();
    all.extend([prefs, at(rva(0x63DB94)), at(rva(0x621514))]);
    let saved = capture(&all);
    for (cars, second, extra) in [
        (3u32, 1u32, 0u32),
        (0, 0, 1),
        (1, 1, 1),
        (9, 2, 0),
        (8, 0, 5),
    ] {
        run_both(
            &mut t,
            &format!("{cars} cars, dword_63DB94 {second}, dword_621514 {extra}"),
            &copy,
            &watched,
            &|| unsafe {
                restore(&all, &saved);
                for n in 0..0x88u32 {
                    let v = match n % 5 {
                        0 => 0,
                        1 => u32::MAX,
                        _ => 0x700 + n,
                    };
                    game_write(rva(0x60A57C) + 4 * n as usize, v);
                }
                game_write(CARS_RVA, cars);
                game_write(rva(0x63DB94), second);
                game_write(rva(0x621514), extra);
                game_write(OTHER_LEVELS_RVA, 1u32);
                game_write(KIND_SLOTS_RVA, 0x100u32);
            },
            &|r| unsafe { init_texture_levels(r, &mut test_part) },
        );
    }
    restore(&all, &saved);
    t.finish()
}
