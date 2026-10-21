//! Item actions the standalone met unported (2026-10-19, the owner's run without RAL.EXE):
//! - `menu::show_intro` (0x47058E): the options' "intro" item: the films played
//!   (`menu::play_films`, eax 1), then back (`menu::go_back`);
//! - `menu::previous_save_page` (0x46D3F2): the championship load page's left arrow: the
//!   page (`dword_75523C`) one less, from 0 round to 9;
//! - `service::open_steering_box`, `service::open_brake_box` (0x46B706, 0x46B71A): the
//!   service area's steering sensitivity and brake bias items: the box switch
//!   (`dword_75AA9C`) turned over and its page (`dword_75AAA0`) 3, or 4;
//! - `menu::view_stage_details` (0x47037E): "view stage info": for a time trial
//!   (`g_eventType` 3) `menu::time_trial_stage_details_loop`, else `menu::view_stage_loop`;
//! - `menu::new_simulation_if_unlocked` (0x4A96BF): with `dword_75ACCC` 1
//!   `menu::new_simulation_loop`, else the refusal's sound (`sound::play_menu_sound` 8) and a
//!   quarter of a second's sleep;
//! - `menu::start_arcade` (0x487CAA): the arcade's level and stage (`g_arcadeLevel`,
//!   `g_arcadeStage`) 0, the first player's units taken (`menu::take_player1_units`) and the
//!   arcade event run (`menu::arcade_event`).
//!
//! In:  nothing (eax, the item's dword, is not read but by `show_intro`, which sets it)
//! Out: the pure ones and those in `pusha` / `popa` every register kept; the others the
//!      registers their last call leaves
//!
//! The calls through `part` (by place in each one's `CALLEES`): in the game the screen loops,
//! the films and the menu sound through their hooks (`call_game`), `go_back`, `sleep`,
//! `take_player1_units` and `arcade_event` by their Rust versions. Those that run a screen
//! have ungated thunks. Shadow mode runs the original alone. Self-test: a copy of each whose
//! calls go to stand-ins, against the Rust version (the switches both ways, the page from 0
//! and not, the event type 3 and not).

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::menu::parts::{Part, copy, dispatch_parted, run_both, test_part};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

const SAVE_PAGE_RVA: usize = rva(0x75523C); // dword_75523C: the save list's page
const BOX_SWITCH_RVA: usize = rva(0x75AA9C); // dword_75AA9C: a service box shown
const BOX_PAGE_RVA: usize = rva(0x75AAA0); // dword_75AAA0: its page
const EVENT_TYPE_RVA: usize = rva(0x755954); // g_eventType
const UNLOCKED_RVA: usize = rva(0x75ACCC); // dword_75ACCC: 1 lets a new simulation start
const ARCADE_LEVEL_RVA: usize = rva(0x7587EC); // g_arcadeLevel
const ARCADE_STAGE_RVA: usize = rva(0x7587F0); // g_arcadeStage

fn g(at: usize) -> u32 {
    unsafe { game_read(at) }
}

fn p(at: usize, v: u32) {
    unsafe { game_write(at, v) }
}

/// A call into the game through its hook (where the Rust version takes closures).
fn hooked(address: usize, r: &mut Regs) {
    unsafe { call_game(r, game_addr(rva(address))) };
}

// ---- the Rust versions ----

const SHOW_INTRO_CALLEES: [usize; 2] = [0x4161CD, 0x471C82]; // play_films, go_back

pub unsafe fn show_intro(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = Regs { eax: 1, ..*entry };
    part(0, &mut r)?;
    part(1, &mut r)?;
    Ok(r)
}

pub fn previous_save_page() {
    // Converted: `rc2k_core::menu::steps::savegame_page`.
    p(
        SAVE_PAGE_RVA,
        rc2k_core::menu::steps::savegame_page(g(SAVE_PAGE_RVA), false),
    );
}

/// The steering (page 3) or brake (page 4) box.
pub fn open_service_box(page: u32) {
    p(BOX_SWITCH_RVA, g(BOX_SWITCH_RVA) ^ 1);
    p(BOX_PAGE_RVA, page);
}

// time_trial_stage_details_loop, view_stage_loop
const VIEW_STAGE_CALLEES: [usize; 2] = [0x48F1F4, 0x47FD37];

pub unsafe fn view_stage_details(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::menu::stage_screens::time_trial_details`.
    let time_trial = rc2k_core::menu::stage_screens::time_trial_details(g(EVENT_TYPE_RVA));
    part(if time_trial { 0 } else { 1 }, &mut r)?;
    Ok(r)
}

// play_menu_sound, sleep, new_simulation_loop
const NEW_SIMULATION_CALLEES: [usize; 3] = [0x40F059, 0x41100C, 0x4A99AA];

pub unsafe fn new_simulation_if_unlocked(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::menu::stage_screens::simulation_unlocked`.
    use rc2k_core::menu::stage_screens::{REFUSED_SLEEP, REFUSED_SOUND, simulation_unlocked};
    if simulation_unlocked(g(UNLOCKED_RVA)) {
        part(2, &mut r)?;
    } else {
        r.eax = REFUSED_SOUND;
        part(0, &mut r)?;
        r.eax = REFUSED_SLEEP;
        part(1, &mut r)?;
    }
    Ok(*entry)
}

// take_player1_units, arcade_event
const START_ARCADE_CALLEES: [usize; 2] = [0x467EA8, 0x487E2C];

pub unsafe fn start_arcade(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    p(ARCADE_LEVEL_RVA, 0);
    p(ARCADE_STAGE_RVA, 0);
    let mut r = *entry;
    part(0, &mut r)?;
    part(1, &mut r)?;
    Ok(r)
}

// ---- in the game ----

fn show_intro_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        0 => hooked(SHOW_INTRO_CALLEES[0], r),
        _ => crate::menu::go_back::go_back(),
    }
    Ok(())
}

fn view_stage_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    hooked(VIEW_STAGE_CALLEES[place as usize], r);
    Ok(())
}

fn new_simulation_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1 => crate::system::sleep::sleep(r.eax),
        _ => hooked(NEW_SIMULATION_CALLEES[place as usize], r),
    }
    Ok(())
}

fn start_arcade_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        0 => crate::menu::take_player1_units::take_player1_units()?,
        _ => {
            *r = unsafe {
                crate::menu::arcade_event::arcade_event(
                    r,
                    &mut crate::menu::arcade_event::game_part,
                )?
            }
        }
    }
    Ok(())
}

// ---- the hooks ----

pub mod show_intro_hook {
    use super::*;
    static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
    pub static HOOK: RawHook =
        RawHook::new("menu::show_intro", rva(0x47058E), thunk, &ORIGINAL).self_test(self_test);
    hook_asm!(
        thunk,
        call_original,
        dispatch = dispatch,
        original = ORIGINAL,
        ungated
    );
    extern "C" fn dispatch(r: &mut Regs) {
        dispatch_parted(
            &HOOK,
            r,
            call_original,
            &|r, part| unsafe { show_intro(r, part) },
            show_intro_part,
        );
    }
    fn self_test() -> (usize, usize) {
        let Some(mut t) = SelfTest::begin(&HOOK) else {
            return (0, 0);
        };
        let copy = match copy(
            0x47058E,
            0x10,
            &[0xB8, 1, 0, 0, 0],
            &[5, 0xA],
            &SHOW_INTRO_CALLEES,
            &[],
        ) {
            Ok(c) => c,
            Err(why) => return t.skip(why),
        };
        run_both(
            &mut t,
            "once",
            &copy,
            &[],
            &|| {},
            Regs::SENTINEL,
            false,
            &|r| unsafe { show_intro(r, &mut test_part) },
        );
        t.finish()
    }
}

/// A pure action's hook: the Rust version on its variables, every register kept; its test
/// a copy against it from each start.
macro_rules! pure_action {
    ($module:ident, $name:literal, $address:expr, $len:expr, $entry:expr, $rust:expr, $vars:expr, $starts:expr) => {
        pub mod $module {
            use super::*;
            static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
            pub static HOOK: RawHook =
                RawHook::new($name, rva($address), thunk, &ORIGINAL).self_test(self_test);
            hook_asm!(
                thunk,
                call_original,
                dispatch = dispatch,
                original = ORIGINAL
            );
            extern "C" fn dispatch(r: &mut Regs) {
                HOOK.count_call();
                if crate::framework::shadow() {
                    unsafe { call_original(r) };
                    return;
                }
                $rust();
            }
            fn self_test() -> (usize, usize) {
                let Some(mut t) = SelfTest::begin(&HOOK) else {
                    return (0, 0);
                };
                let copy = match copy($address, $len, &$entry, &[], &[], &[]) {
                    Ok(c) => c,
                    Err(why) => return t.skip(why),
                };
                let vars: Vec<Region> = $vars.iter().map(|&v| at(v)).collect();
                let saved = capture(&vars);
                for start in $starts {
                    let set = || {
                        for (&v, &x) in $vars.iter().zip(start.iter()) {
                            p(v, x);
                        }
                    };
                    run_both(
                        &mut t,
                        &format!("from {start:x?}"),
                        &copy,
                        &vars,
                        &set,
                        Regs::SENTINEL,
                        false,
                        &|r| {
                            $rust();
                            Ok(*r)
                        },
                    );
                }
                restore(&vars, &saved);
                t.finish()
            }
        }
    };
}

pure_action!(
    previous_save_page_hook,
    "menu::previous_save_page",
    0x46D3F2,
    0x15,
    [0x60, 0xFF, 0x0D, 0x3C, 0x52, 0x75, 0x00],
    previous_save_page,
    [SAVE_PAGE_RVA],
    [[0u32], [1], [5], [9], [0xFFFF_FFFF]]
);
pure_action!(
    open_steering_box_hook,
    "service::open_steering_box",
    0x46B706,
    0x14,
    [0x60, 0x83, 0x35, 0x9C, 0xAA, 0x75, 0x00, 0x01],
    || open_service_box(3),
    [BOX_SWITCH_RVA, BOX_PAGE_RVA],
    [[0u32, 0], [1, 7], [2, 3]]
);
pure_action!(
    open_brake_box_hook,
    "service::open_brake_box",
    0x46B71A,
    0x14,
    [0x60, 0x83, 0x35, 0x9C, 0xAA, 0x75, 0x00, 0x01],
    || open_service_box(4),
    [BOX_SWITCH_RVA, BOX_PAGE_RVA],
    [[0u32, 0], [1, 7], [2, 4]]
);

pub mod view_stage_details_hook {
    use super::*;
    static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
    pub static HOOK: RawHook =
        RawHook::new("menu::view_stage_details", rva(0x47037E), thunk, &ORIGINAL)
            .self_test(self_test);
    hook_asm!(
        thunk,
        call_original,
        dispatch = dispatch,
        original = ORIGINAL,
        ungated
    );
    extern "C" fn dispatch(r: &mut Regs) {
        dispatch_parted(
            &HOOK,
            r,
            call_original,
            &|r, part| unsafe { view_stage_details(r, part) },
            view_stage_part,
        );
    }
    fn self_test() -> (usize, usize) {
        let Some(mut t) = SelfTest::begin(&HOOK) else {
            return (0, 0);
        };
        let copy = match copy(
            0x47037E,
            0x15,
            &[0x83, 0x3D, 0x54, 0x59, 0x75, 0x00, 0x03],
            &[9, 0xF],
            &VIEW_STAGE_CALLEES,
            &[],
        ) {
            Ok(c) => c,
            Err(why) => return t.skip(why),
        };
        let vars = [at(EVENT_TYPE_RVA)];
        let saved = capture(&vars);
        for kind in [3u32, 1, 0] {
            run_both(
                &mut t,
                &format!("event type {kind}"),
                &copy,
                &vars,
                &|| p(EVENT_TYPE_RVA, kind),
                Regs::SENTINEL,
                false,
                &|r| unsafe { view_stage_details(r, &mut test_part) },
            );
        }
        restore(&vars, &saved);
        t.finish()
    }
}

pub mod new_simulation_if_unlocked_hook {
    use super::*;
    static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
    pub static HOOK: RawHook = RawHook::new(
        "menu::new_simulation_if_unlocked",
        rva(0x4A96BF),
        thunk,
        &ORIGINAL,
    )
    .self_test(self_test);
    hook_asm!(
        thunk,
        call_original,
        dispatch = dispatch,
        original = ORIGINAL,
        ungated
    );
    extern "C" fn dispatch(r: &mut Regs) {
        dispatch_parted(
            &HOOK,
            r,
            call_original,
            &|r, part| unsafe { new_simulation_if_unlocked(r, part) },
            new_simulation_part,
        );
    }
    fn self_test() -> (usize, usize) {
        let Some(mut t) = SelfTest::begin(&HOOK) else {
            return (0, 0);
        };
        let copy = match copy(
            0x4A96BF,
            0x27,
            &[0x60, 0x83, 0x3D, 0xCC, 0xAC, 0x75, 0x00, 0x01],
            &[0xF, 0x19, 0x20],
            &NEW_SIMULATION_CALLEES,
            &[],
        ) {
            Ok(c) => c,
            Err(why) => return t.skip(why),
        };
        let vars = [at(UNLOCKED_RVA)];
        let saved = capture(&vars);
        for unlocked in [1u32, 0, 2] {
            run_both(
                &mut t,
                &format!("unlocked {unlocked}"),
                &copy,
                &vars,
                &|| p(UNLOCKED_RVA, unlocked),
                Regs::SENTINEL,
                false,
                &|r| unsafe { new_simulation_if_unlocked(r, &mut test_part) },
            );
        }
        restore(&vars, &saved);
        t.finish()
    }
}

pub mod start_arcade_hook {
    use super::*;
    static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
    pub static HOOK: RawHook =
        RawHook::new("menu::start_arcade", rva(0x487CAA), thunk, &ORIGINAL).self_test(self_test);
    hook_asm!(
        thunk,
        call_original,
        dispatch = dispatch,
        original = ORIGINAL,
        ungated
    );
    extern "C" fn dispatch(r: &mut Regs) {
        dispatch_parted(
            &HOOK,
            r,
            call_original,
            &|r, part| unsafe { start_arcade(r, part) },
            start_arcade_part,
        );
    }
    fn self_test() -> (usize, usize) {
        let Some(mut t) = SelfTest::begin(&HOOK) else {
            return (0, 0);
        };
        let copy = match copy(
            0x487CAA,
            0x1F,
            &[0xC7, 0x05, 0xEC, 0x87, 0x75, 0x00, 0x00, 0x00, 0x00, 0x00],
            &[0x14, 0x19],
            &START_ARCADE_CALLEES,
            &[],
        ) {
            Ok(c) => c,
            Err(why) => return t.skip(why),
        };
        let vars = [at(ARCADE_LEVEL_RVA), at(ARCADE_STAGE_RVA)];
        let saved = capture(&vars);
        run_both(
            &mut t,
            "once",
            &copy,
            &vars,
            &|| {
                p(ARCADE_LEVEL_RVA, 3);
                p(ARCADE_STAGE_RVA, 5);
            },
            Regs::SENTINEL,
            false,
            &|r| unsafe { start_arcade(r, &mut test_part) },
        );
        restore(&vars, &saved);
        t.finish()
    }
}

/// The seven, for the registry.
pub static HOOKS: [&dyn crate::framework::GameHook; 7] = [
    &show_intro_hook::HOOK,
    &previous_save_page_hook::HOOK,
    &open_steering_box_hook::HOOK,
    &open_brake_box_hook::HOOK,
    &view_stage_details_hook::HOOK,
    &new_simulation_if_unlocked_hook::HOOK,
    &start_arcade_hook::HOOK,
];
