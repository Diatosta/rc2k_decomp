//! `physics::find_clear_place` (0x43BDD1): the work car moved to a place where nothing
//! touches it, after a reset. `dword_614C28` = 0x5A; its speed times `flt_613FDC`
//! (`physics::scale_vector`); `physics::restore_saved_car`;
//! `physics::lowest_body_point`, the depth into `dword_614118`. Then tries n = 0x70 +
//! (`unk_719FE0` & 0xF) .. 0xFF: `dword_614738` (counted by `restore_saved_car`) = 0, the
//! place moved by 2^((n >> 4) + 1) x (`dword_614014[n & 0xF]`, `dword_614054[n & 0xF]`)
//! (x, z; `imul`, low dwords), `physics::probe_ground`, `physics::crossed_edge`,
//! `physics::wall_contacts`, `physics::car_collisions`, `physics::face_contacts`; a try
//! that restored nothing ends it. None: `physics::reset_car`. Found:
//! `lowest_body_point`, a depth below 0 taken off the height (+0x800); "porky", n, the
//! first depth and the last logged (level 1, in `pusha` / `popa`); `dword_61473C` + 1;
//! `physics::limit_motion`.
//!
//! Not the physics step's loop (that is `sub_43B635`'s caller): a bounded search inside
//! one step, so the hook is gated as usual.
//!
//! In:  nothing
//! Out: the registers as the steps and calls leave them (ebp the try, kept around each
//!      try's calls); no flags read (the caller returns)
//!
//! Caller: `sub_43B635` (0x43B8DB).
//! Refuses (before anything) a probe count of 0 or past 0x40; what its calls refuse
//! (everything written put back, the faces touched included).
//! Shadow mode: the Rust version with `g_logEnabled` 0 (its log lines not written
//! twice), then the original; `regions()` and the faces touched compared.
//! Self-test (in a race; skipped otherwise; `g_logEnabled` 0 for it): the original and
//! the Rust version on the stage (everything written saved and put back), the car where
//! it is and moved onto object faces (so tries fail and the search goes on).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::physics::face_contact::{
    Touched, face_bytes, faces_region, move_car_onto, object_faces, put_back_faces,
};
use crate::physics::flow::{a, g, p};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::{extended, rounded};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, restore};
use rc2k_core::physics::clear_place;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::find_clear_place", rva(0x43BDD1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LOG_ENABLED: usize = 0x4E0094; // g_logEnabled
const PORKY: usize = 0x4DBAFB; // aPorky

/// One try's calls (0x43BE4D .. 0x43BE61).
fn try_place(r: &mut Regs, touched: &mut Touched) -> Result<(), Refused> {
    unsafe { crate::physics::probe_ground::probe_ground(r) }?;
    crate::physics::crossed_edge::check()?;
    crate::physics::crossed_edge::steps(r)?;
    crate::physics::wall_contacts::check()?;
    crate::physics::wall_contacts::steps(r)?;
    unsafe { crate::physics::car_collisions::car_collisions(r) }?.apply(r);
    crate::physics::face_contacts_around::steps(r, touched)
}

/// The steps, not guarded.
// Converted: `rc2k_core::physics::clear_place` (the tries' contacts are the hooks').
pub fn steps(r: &mut Regs, touched: &mut Touched) -> Result<(), Refused> {
    p(0x614C28, clear_place::TRY_MARK);
    let factor = extended(g(0x613FDC));
    unsafe { crate::physics::scale_vector::scale_vector(a(0x71A170), factor) };
    p(0x62400C, rounded(factor));
    unsafe { crate::physics::restore_saved_car::restore_saved_car() }?;
    unsafe { crate::physics::lowest_body_point::lowest_body_point(r) }?;
    r.eax = g(0x6D3068);
    p(0x614118, r.eax);
    let found = clear_place::tries(clear_place::first_try(g(0x719FE0)), |n| {
        r.ebp = n;
        p(0x614738, 0);
        let m = clear_place::try_move(n);
        r.ebx = m.entry;
        r.ecx = m.shift;
        for (table, place) in [(0x614014, 0x71A10C), (0x614054, 0x71A114)] {
            (r.eax, r.edx) = clear_place::move_part(m.step, g(table + 4 * r.ebx as usize));
            p(place, g(place).wrapping_add(r.eax));
        }
        r.eax = m.step;
        try_place(r, touched)?;
        r.ebp = n;
        Ok::<_, Refused>(g(0x614738) == 0)
    })?;
    if found.is_none() {
        r.ebp = clear_place::TRIES_END;
        *r = unsafe {
            crate::physics::reset_car::reset_car(r, &mut crate::physics::reset_parts::game_part)
        }?;
        return Ok(());
    }
    unsafe { crate::physics::lowest_body_point::lowest_body_point(r) }?;
    let depth = g(0x6D3068);
    if let Some(height) = clear_place::lifted(g(0x71A110), depth) {
        r.eax = depth;
        p(0x71A110, height);
    }
    unsafe {
        crate::logfile::log_write::log_write(a(PORKY) as *const u8, false, 1);
        crate::logfile::log_int::log_int(r.ebp as i32, false, 1);
        crate::logfile::log_hex8::log_hex8(g(0x614118), false, 1);
        crate::logfile::log_hex8::log_hex8(g(0x6D3068), true, 1);
    }
    p(0x61473C, g(0x61473C).wrapping_add(1));
    unsafe { crate::physics::limit_motion::limit_motion() }.apply(r);
    Ok(())
}

/// The Rust version.
pub unsafe fn find_clear_place(entry: &Regs) -> Result<Left, Refused> {
    crate::physics::wall_contacts::check()?;
    let all = regions();
    let saved = capture(&all);
    let (mut r, mut touched) = (*entry, Touched::new());
    match steps(&mut r, &mut touched) {
        Ok(()) => Ok(left_of(&r)),
        Err(e) => {
            restore(&all, &saved);
            put_back_faces(&touched);
            Err(e)
        }
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { find_clear_place(r) }
}

/// Everything it and its calls write but the faces.
pub fn regions() -> Vec<Region> {
    let mut all = crate::physics::face_contact::regions();
    all.extend(crate::physics::crossed_edge::regions());
    all.extend(crate::physics::car_collisions::regions());
    all.extend(crate::physics::probe_ground::regions(
        &crate::physics::ground::Ground::game(),
    ));
    all.extend(crate::physics::limit_motion::regions());
    all.extend(crate::physics::reset_parts::regions());
    all.extend([
        (a(0x614C28), 4),
        (a(0x614118), 4),
        (a(0x614738), 8),
        (a(0x6D3068), 12),
        (a(0x71A10C), 12),
    ]);
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    if !crate::framework::shadow() {
        crate::service::harness::dispatch_state(&HOOK, r, call_original, &[], &rust);
        return;
    }
    HOOK.count_call();
    if let Err(e) = crate::physics::wall_contacts::check() {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        return;
    }
    let all = regions();
    let before = capture(&all);
    let logging = g(LOG_ENABLED);
    p(LOG_ENABLED, 0);
    let (mut mine_regs, mut touched) = (*r, Touched::new());
    let result = steps(&mut mine_regs, &mut touched);
    p(LOG_ENABLED, logging);
    if let Err(e) = result {
        restore(&all, &before);
        put_back_faces(&touched);
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        return;
    }
    let left = left_of(&mine_regs);
    let mine = (capture(&all), face_bytes(&touched));
    restore(&all, &before);
    put_back_faces(&touched);
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = (capture(&all), face_bytes(&touched));
    if theirs != mine || !left.matches(r, &o) {
        crate::screens::harness::mismatch_unless_disturbed(&HOOK, || {
            format!(
                "regions or faces differ: {}; regs_ok={}",
                theirs != mine,
                left.matches(r, &o)
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = crate::framework::SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !crate::physics::move_slot::in_race() || !(1..=0x40).contains(&g(0x71A238)) {
        return t.skip("not in a race (no stage)");
    }
    let mut all = regions();
    all.extend([faces_region(), (a(LOG_ENABLED), 4), (a(0x71B400), 4)]);
    let saved = capture(&all);
    let mut moves = vec![None];
    moves.extend(object_faces(4).into_iter().map(Some));
    for moved in moves {
        let set = || {
            restore(&all, &saved);
            p(LOG_ENABLED, 0);
            if let Some(k) = moved {
                move_car_onto(k);
            }
        };
        check_state(
            &mut t,
            &format!("car moved onto {moved:?}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
