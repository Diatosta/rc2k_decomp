//! `menu::class_rally_results_loop` (0x466858): the loop of the third results screen after a
//! stage, the rally's standings so far for the player's class.
//!
//! Screen: the owner's screenshots of 2026-10-10 and 2026-10-11, from visits the Rust version
//! ran (hook.log): title "QUADRO DE LIDERANÇA CLASSE" (after the rally's last stage or a
//! retirement "LIDER DA CLASSIFICAÇÃO FINAL DE CLASSES"), the rally's name, rows with the
//! total time and gap, or "abandonou" / "não terminou". Named `screen_466858_loop`, then
//! `rally_results_loop`.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`): before the loop it sets the string of the menu's first item to 0x138 if
//! `dword_75AADC` is 1 and to 0x139 otherwise, as the stage results screen does;
//! its background comes from `menu::load_background`, after which a set carry flag is fatal
//! error 0x80712345; leaving is asked about, and a "no" sets the screen up again.
//! One caller, `sub_4A3317`. Ungated thunk; shadow mode runs the original alone; each real call writes a
//! line to hook.log. Self-test: `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::class_stage_results_loop::LOAD_BACKGROUND;
use crate::menu::run_draw_list::DRAW_LIST_RVA;
use crate::menu::screen_loop::{
    FRAME_CALLBACK, List, Original, Screen, Step, check_screen, dispatch_screen,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::class_rally_results_loop",
    rva(0x466858),
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

// ---- game data ----

const MENU_RVA: usize = rva(0x74FC60); // off_74FC60
const FIRST_ITEM_STRING_RVA: usize = MENU_RVA + 4 + 0x14;
const WHICH_TITLE_RVA: usize = rva(0x75AADC); // dword_75AADC
const BACKGROUND_RVA: usize = rva(0x73253A); // "files\g\graphics\screens\results1.png"

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_first_item(g: &mut Regs) {
    unsafe {
        g.eax = if game_read::<u32>(WHICH_TITLE_RVA) == 1 {
            0x138
        } else {
            0x139
        };
        g.edi = game_addr(MENU_RVA) as u32 + 4;
        game_write(FIRST_ITEM_STRING_RVA, g.eax);
    }
}

pub const SCREEN: Screen = Screen {
    menu: MENU_RVA,
    saved_item: rva(0x747788), // dword_747788
    lists: &[
        List {
            pointer: rva(0x755504),
            table: rva(0x7590DA),
            run: false,
        },
        List {
            pointer: DRAW_LIST_RVA,
            table: rva(0x758E2A),
            run: false,
        },
        List {
            pointer: rva(0x7554FC),
            table: rva(0x75929E),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FRAME_CALLBACK,
    later_frame_callback: None,
    before: &[Step::Do(set_first_item)],
    set_up_first: &[],
    prepare: &[],
    background: &[
        Step::EaxAddr(BACKGROUND_RVA),
        Step::CallOrFatal(LOAD_BACKGROUND, 0x8071_2345),
    ],
    after_first_frame: &[],
    confirm: Some(&[]),
    after: &[],
    leave_at_once: None,
    also_writes: &[FIRST_ITEM_STRING_RVA],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x466858),
    len: 0x2C9,
    entry_bytes: &[0x60, 0xB8, 0x39, 0x01, 0x00, 0x00, 0x83, 0x3D],
    calls: &[0xAC, 0xBD, 0xC2, 0x111, 0x116, 0x15E, 0x1BC, 0x25E, 0x283],
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
