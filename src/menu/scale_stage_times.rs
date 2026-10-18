//! `menu::scale_stage_times` (0x46354E): the stage's four record times (`dword_75AAE8` ..
//! `dword_75AAF4`) scaled for the section (`section` held to 0 .. 5): the last three by
//! `flt_60463C`[section], the first by `flt_604654`[section] (`fild`, `fmul`, `fistp`; the
//! two factors kept in `flt_71C3D8` / `flt_71C3DC`); then logged ("stage times info", the
//! section, the record's +0x40, "a7x" and the first factor, "a8x" and the second, "times
//! a8-a5" and the four times).
//!
//! In:  edi the record
//! Out: eax the section held (no `pusha`); the rest kept
//!
//! Caller: `menu::calc_cpu_times`. Logs through the `logfile::` Rust versions (level 1).
//! Shadow mode runs the original alone (it logs). Self-test (unpaused: it logs): the
//! original and the Rust version on a record of the test's, times and sections by lot.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::scale_stage_times", rva(0x46354E), thunk, &ORIGINAL)
    .self_test(self_test)
    .self_test_unpaused();

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const TIMES_RVA: usize = rva(0x75AAE8); // dword_75AAE8 .. dword_75AAF4
const KEPT_RVA: [usize; 2] = [rva(0x71C3D8), rva(0x71C3DC)]; // flt_71C3D8, flt_71C3DC
const FACTORS_RVA: [usize; 2] = [rva(0x60463C), rva(0x604654)]; // flt_60463C, flt_604654: 6
const TEXTS_RVA: [usize; 4] = [rva(0x4DDF93), rva(0x4DDFA4), rva(0x4DDFA8), rva(0x4DDFAC)];
const EXTRA: [usize; 1] = [SECTION_RVA];

static mut RECORD: [u32; 0x20] = [0; 0x20];
const CASES: u32 = 4;

fn test_entry(_case: u32) -> Regs {
    Regs {
        edi: (&raw const RECORD) as u32,
        ..Regs::SENTINEL
    }
}

fn prepare(case: u32) {
    unsafe {
        game_write(SECTION_RVA, [0u32, 3, 9, 0xFFFF_FFFE][case as usize]);
        (&raw mut RECORD).cast::<u32>().add(0x10).write(0x40 + case);
        for k in 0..4 {
            game_write(
                TIMES_RVA + 4 * k,
                0x1_0000 * (k as u32 + 1) + case * 0x3333 + 7,
            );
        }
    }
}

/// The section held to 0 .. 5.
pub fn held_section() -> u32 {
    (unsafe { game_read::<i32>(SECTION_RVA) }).clamp(0, 5) as u32
}

/// The four times as scaled (see the header), and the two factors.
pub fn scaled(times: [u32; 4], section: u32) -> ([u32; 4], [u32; 2]) {
    let f: [u32; 2] = FACTORS_RVA.map(|r| unsafe { game_read(r + 4 * section as usize) });
    let mut out = times;
    for (k, t) in out.iter_mut().enumerate() {
        *t = crate::math::x87::int_times_to_int(*t as i32, f[usize::from(k == 0)]) as u32;
    }
    (out, f)
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn scale_stage_times(entry: &Regs) -> Result<Regs, Refused> {
    use crate::logfile::{log_float::log_float, log_int::log_int, log_write::log_write};
    let section = held_section();
    unsafe {
        let times: [u32; 4] = game_read(TIMES_RVA);
        let (times, f) = scaled(times, section);
        game_write(KEPT_RVA[0], f[0]);
        game_write(KEPT_RVA[1], f[1]);
        game_write(TIMES_RVA, times);
        let text = |n: usize| game_addr(TEXTS_RVA[n]) as *const u8;
        log_write(text(0), false, 1);
        log_int(section as i32, false, 1);
        log_int(get(entry.edi + 0x40) as i32, false, 1);
        log_write(text(1), false, 1);
        log_float(f32::from_bits(f[0]), 2, false, 1);
        log_write(text(2), false, 1);
        log_float(f32::from_bits(f[1]), 2, false, 1);
        log_write(text(3), false, 1);
        for (k, &t) in times.iter().enumerate() {
            log_int(t as i32, k == 3, 1);
        }
    }
    Ok(Regs {
        eax: section,
        ..*entry
    })
}

fn regions() -> Vec<Region> {
    vec![
        (game_addr(TIMES_RVA) as u32, 16),
        at(KEPT_RVA[0]),
        at(KEPT_RVA[1]),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { scale_stage_times(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    // It logs: in shadow mode the original alone (the self-test compares the two).
    if crate::framework::shadow() {
        HOOK.count_call();
        unsafe { call_original(r) };
        return;
    }
    let regions = regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Unpaused: frames drawn meanwhile may see the tables this test changes.
    let _disturbing = crate::screens::harness::Disturbing::begin();
    let regions = regions();
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { scale_stage_times(&test_entry(0)) }.is_err();
    restore(&vars, &saved);
    if refused {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            test_entry(case),
            &|| {
                restore(&vars, &saved);
                prepare(case);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
