//! `service::service_area_loop` (0x47284A): the service area screen, from entering it to
//! leaving it. The first of the game's 83 screen loops to be ported; the loop itself is
//! `menu::screen_loop`, and this file says what is the service area's own (`SCREEN`).
//!
//! In:  nothing
//! Out: every register preserved
//! It empties the key buffer (six `take_key`), then:
//!
//! Set-up (again after every action that does not leave the screen): the highlight comes
//! back from `dword_74777C`, the scene is prepared (`sub_46F7FD`, a few of its variables,
//! `sub_40F7A4`), the screen's menu `off_74D24C` and its four function lists are put in place
//! (the list at 0x759436 is run once, here), the background is loaded (`sub_470F55`), one
//! frame is drawn (`menu::draw_frame`, then `screens::run_frame` with the callback
//! `sub_474D4B`), and the time is noted in `dword_5189A0`.
//!
//! Every frame: `menu::update_blink`; leave if the game is quitting; `screens::run_frame`;
//! `dword_613830` = milliseconds since set-up; then `menu::handle_keys`:
//!   1  next frame
//!   2  (Escape) leave
//!   3  (Enter) the highlighted item's action runs (`menu::run_item_action`, with the
//!      highlight saved to `dword_74777C` before and taken back from it after). Then:
//!      leave if `dword_755569` or `dword_755980` is 1, or if `dword_755561` is 1 (which is
//!      cleared); else next frame if `dword_755A38` is 2; else leave if `dword_75555D` is 1
//!      (clearing `dword_755561`); else set up again.
//!   else leave
//! The same action check is made at the top of a frame whose `dword_7556A4` is 0x14D.
//!
//! Leaving: if `dword_758806` is 1, `sub_46FA90` asks whether to; unless the answer
//! (`g_yesNoAnswer`) is 1, the background is loaded again and the screen is set up again.
//! Otherwise the highlight is saved, `dword_7554E4` = 1, `sub_40F826`, `dword_75AAE0` = 0.
//!
//! Wherever the highlight is saved or taken back, `dword_755A3C` and `dword_755561` are
//! cleared if `dword_7558D8` and `dword_755A3C` are both 1.
//!
//! The registers are carried through as the original has them, because the functions it
//! calls get them: `handle_keys` reads ah on Enter, which here is a byte of the elapsed time.
//!
//! Callers: `sub_48E093` (time trial) and `sub_4A15E3` (championship).
//! Seven of its callees are not ported and are called with `call_game`: the four named
//! under "game data" below, and `menu::draw_frame`, `screens::run_frame` and
//! `menu::confirm_leaving` in the shared loop. Ungated thunk; shadow mode runs the original
//! alone; each real call writes a line to hook.log saying which version ran.
//! Manual patch with a private copy (0x360 bytes, 20 calls relocated): a call sits in the
//! first bytes. Self-test: `screen_loop::check_screen`, a redirected copy whose calls all go
//! to stand-ins that follow a script.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::run_draw_list::DRAW_LIST_RVA;
use crate::menu::screen_loop::{
    FRAME_CALLBACK, List, Original, Screen, Step, TAKE_KEY, check_screen, dispatch_screen,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::service_area_loop",
    rva(0x47284A),
    thunk,
    &ORIGINAL,
)
.private_copy(FUNCTION_LEN)
.relocate_calls(&CALLS)
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- game data ----

const SETUP_LOADED_RVA: usize = rva(0x75AAE0); // dword_75AAE0: 1 = `load_setup_values` does nothing
const LEFT_RVA: usize = rva(0x7554E4); // dword_7554E4: 0 on the screen, 1 after it
const PREPARE_SCENE: usize = rva(0x46F7FD); // scene::prepare_menu_scene
const SUB_40F7A4: usize = rva(0x40F7A4);
const LOAD_BACKGROUND: usize = rva(0x470F55); // service::load_background: the picture for the car
const SUB_40F826: usize = rva(0x40F826);

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: rva(0x74D24C),       // off_74D24C
    saved_item: rva(0x74777C), // g_serviceSavedItem
    lists: &[
        List {
            pointer: rva(0x755504), // dword_755504
            table: rva(0x759082),   // off_759082: the scene
            run: false,
        },
        List {
            pointer: DRAW_LIST_RVA,
            table: rva(0x758D6E), // off_758D6E: `handle_edit_keys`
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x759436),   // off_759436: `load_setup_values`
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // dword_7554FC
            table: rva(0x75924A),   // off_75924A: `draw_service_area`
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FRAME_CALLBACK,
    // The key buffer is emptied first.
    before: &[
        Step::Call(TAKE_KEY),
        Step::Call(TAKE_KEY),
        Step::Call(TAKE_KEY),
        Step::Call(TAKE_KEY),
        Step::Call(TAKE_KEY),
        Step::Call(TAKE_KEY),
        Step::Eax(0),
        Step::Write(SETUP_LOADED_RVA, 0),
    ],
    set_up_first: &[Step::Write(LEFT_RVA, 0)],
    prepare: &[
        Step::Write(rva(0x623FB0), 0),     // dword_623FB0
        Step::Write(rva(0x623FB4), 0),     // dword_623FB4
        Step::Write(rva(0x624080), 0x100), // dword_624080
        Step::Call(PREPARE_SCENE),
        Step::Write(rva(0x6241DC), 0),        // dword_6241DC
        Step::Write(rva(0x6241E0), 800),      // dword_6241E0
        Step::WriteF32(rva(0x6241EC), 0.0),   // flt_6241EC
        Step::WriteF32(rva(0x6241F0), 800.0), // flt_6241F0
        Step::Call(SUB_40F7A4),
    ],
    background: &[Step::Call(LOAD_BACKGROUND)],
    after_first_frame: &[Step::Write(SETUP_LOADED_RVA, 1)],
    confirm: Some(&[Step::Call(LOAD_BACKGROUND)]),
    after: &[
        Step::Write(LEFT_RVA, 1),
        Step::Call(SUB_40F826),
        Step::Write(SETUP_LOADED_RVA, 0),
    ],
    leave_at_once: None,
    also_writes: &[],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

// ---- self-test ----

const FUNCTION_LEN: usize = 0x360;
const CALLS: [usize; 20] = [
    0x01, 0x06, 0x0B, 0x10, 0x15, 0x1A, 0x95, 0xC6, 0xF5, 0x120, 0x12B, 0x130, 0x17F, 0x18E, 0x1D6,
    0x234, 0x2D6, 0x2FB, 0x309, 0x34F,
];
const THE_ORIGINAL: Original = Original {
    function: rva(0x47284A),
    len: FUNCTION_LEN,
    entry_bytes: &[0x60, 0xE8, 0x45, 0x57, 0xF9, 0xFF, 0xE8, 0x40],
    calls: &CALLS,
};

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Err(why) = check_screen(&mut t, &SCREEN, &THE_ORIGINAL) {
        return t.skip(why);
    }
    t.finish()
}
