//! `marks::load_splines` (0x4B96CD): in step 1, `files\m\spline.dat` read once and a spline
//! made for each of the settings' objects.
//!
//! In:  nothing
//! Out: esi the settings, the rest as the last `marks::make_spline` leaves them
//!
//! See `marks/splines.rs`. One caller, `sub_4C1235` (`init_marks`' step 1). Its once-only
//! mark (`g_marksSplinesLoaded`) holds `loc_4B9706` after the first call. Refuses a mark
//! neither 0 nor that, a kind of 0x22 or more, more than 19 objects, and what
//! `make_spline` refuses for any object made, before writing anything. Self-test: the
//! original and the Rust version on the scene of `marks::measure_lines` (0, 1, 2 and 9
//! objects; the file read and not), the game's file and tables put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::marks::lines::{TEST_OBJECTS, TestScene};
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::load_splines", rva(0x4B96CD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let mut r = table_regions();
    r.extend(work_regions());
    r.extend([
        at(OBJECT_RVA),
        at(PLACE_RVA),
        at(OFFSET_RVA),
        at(LOADED_RVA),
        at(MADE_RVA),
        (game_addr(CHOSEN_RVA) as u32, 20),
        (game_addr(FILE_RVA) as u32, FILE_LEN as usize),
        at(crate::files::read_whole_file::BYTES_READ_RVA),
    ]);
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let resume = game_addr(RESUME_RVA) as u32;
    Ok(left_of(&unsafe {
        load_splines(r, resume, &mut crate::race::level_init::game_part)
    }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let kind = [at(crate::marks::kind_values::KIND_RVA)];
    let saved = (capture(&regions), capture(&kind));
    let scene = TestScene::new();
    let settings = scene.settings.as_ptr() as u32;
    let resume = game_addr(RESUME_RVA) as u32;
    for objects in [0u32, 1, 2, 9] {
        for loaded in [0u32, resume] {
            check_state(
                &mut t,
                &format!("{objects} objects, loaded {}", loaded != 0),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    restore(&regions, &saved.0);
                    scene.fill(objects + 3, 1);
                    let at = |o: u32| (settings + o) as *mut u32;
                    at(0x26B8).write_unaligned(objects);
                    for k in 0..objects {
                        at(0x26BC + 4 * k).write_unaligned(TEST_OBJECTS[k as usize % 2]);
                    }
                    game_write(crate::marks::kind_values::KIND_RVA, objects % 5);
                    game_write(LOADED_RVA, loaded);
                },
                &regions,
                &rust,
            );
        }
    }
    unsafe { game_write(LOADED_RVA, 1u32) };
    t.check(
        unsafe { load_splines(&Regs::SENTINEL, resume, &mut |_, _| Ok(())) }.is_err(),
        || "a mark of 1: the Rust version went on".into(),
    );
    drop(scene);
    restore(&regions, &saved.0);
    restore(&kind, &saved.1);
    t.finish()
}
