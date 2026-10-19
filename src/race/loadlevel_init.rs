//! `race::loadlevel_init` (0x44426F): the stage's loading proper ("LOADLEVEL_INIT"), its
//! 27 steps each logged, indented and marked on the loading bar. See `race/loadlevel.rs`.
//!
//! In:  nothing
//! Out: the registers as the last call (`scene::find_object_of_kind`) leaves them
//!
//! One caller, `race::load_stage` (0x4440B1). Its calls are the Rust versions but two,
//! which go through the game (`call_game`, section 15): init mark (`sub_456EA7`) and the
//! loading bar's callback (`sub_4467C7`, a `call esi` on `dword_623704`). Its caller reads no flags after it (`pushf` follows). Shadow mode
//! runs the original alone (it logs and loads the stage). Self-test: a copy whose 57
//! calls go to stand-ins that note them (the log and the loading bar too, the callback's
//! address patched), its 59 references to `g_logIndent`, `g_logEnabled`, `g_inRace` and
//! the callback the test's, against the Rust version on the same: editor or not, the
//! stages that flip the direction, the refresh flags, with and without `dword_64BB18`.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::loadlevel::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::loadlevel_init", rva(0x44426F), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { loadlevel_init(r, LoadVars::game(), &mut game_part) } {
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

const CALLS: [usize; 57] = [
    0x7, 0x9F, 0xAE, 0xC3, 0xD2, 0x118, 0x127, 0x131, 0x15A, 0x169, 0x192, 0x1A1, 0x1CA, 0x1D9,
    0x202, 0x211, 0x23A, 0x249, 0x272, 0x281, 0x2AA, 0x2B9, 0x2E2, 0x2F1, 0x32E, 0x33D, 0x366,
    0x381, 0x3AA, 0x3B9, 0x3E2, 0x401, 0x430, 0x43F, 0x4A0, 0x4AF, 0x4D8, 0x4E7, 0x510, 0x51F,
    0x548, 0x557, 0x580, 0x58F, 0x5D6, 0x5E5, 0x5FA, 0x609, 0x627, 0x636, 0x64B, 0x65A, 0x66F,
    0x67E, 0x688, 0x69C, 0x6CA,
];

fn self_test() -> (usize, usize) {
    use crate::audio::test_copy::{Copy, make};
    use crate::install::stand_ins::{set_handler, stand_in};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The test's indent, log switch and `g_inRace`.
    let vars = Box::new([0u32; 3]);
    let base = vars.as_ptr() as u32;
    let test = LoadVars {
        indent: base,
        log_enabled: base + 4,
        in_race: base + 8,
        bar: stand_in(23) as u32,
    };
    let copy = match make(
        &HOOK,
        &Copy {
            address: 0x44426F,
            len: 0x6D1,
            entry: &[0x60, 0x8D, 0x05, 0x6E, 0xBE, 0x4D, 0x00],
            calls: &CALLS,
            jumps_out: &[],
        },
        &redirect,
        &[
            (0x4E00A0, test.indent),
            (0x4E0094, test.log_enabled),
            (0x5C5CEC, test.in_race),
            (0x4467C7, test.bar),
        ],
        59,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(crate::race::level_init::note);
    let mut regions: Vec<Region> = regions();
    regions.push((base, 12));
    let saved = capture(&regions);
    for (editor, direction, stage, flags, no_place) in [
        (0u32, 0u32, 0x15u32, [0u32, 0u32], 0u32),
        (1, 1, 0x1F, [3, 1], 0),
        (0, 0, 0x22, [2, 0xFFFF_FFFF], 1),
        (5, 7, 0x51, [1, 2], 0),
        (0, 1, 0x30, [0xFFFF_FFFF, 0], 2),
    ] {
        crate::race::race_cars::run_cars(
            &mut t,
            &format!(
                "editor {editor}, direction {direction}, stage {stage:#x}, flags {flags:x?}, no place {no_place}"
            ),
            &copy,
            &regions,
            Regs::SENTINEL,
            &|| unsafe {
                prepare(editor, direction, stage, flags, no_place);
                let v = base as *mut u32;
                v.write(0x30);
                v.add(1).write(1);
                v.add(2).write(0);
            },
            &|r| unsafe {
                loadlevel_init(r, test, &mut |c, r| {
                    crate::race::level_init::note(place_of(c), r);
                    Ok(())
                })
            },
        );
    }
    restore(&regions, &saved);
    drop(vars);
    t.finish()
}
