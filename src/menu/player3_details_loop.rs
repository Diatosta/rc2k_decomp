//! `menu::player3_details_loop` (0x477C10): the loop of the third player's details screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `pdet1.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; it has a step of its own once
//! a frame; after an action it is left or set up again, nothing else; the steps that are more
//! than a call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "PORMENORES DO JOGADOR 3", not yet
//! a visit.
//! Callers: none in code: it is reached through a table. Ungated thunk; shadow mode runs the
//! original alone; each real call writes a line to hook.log. Self-test:
//! `screen_loop::check_screen`. Manual patch with a private copy (0x287 bytes, 10 calls and
//! jumps relocated): a call or a jump target sits in the first bytes.

use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::menu::screen_loop::{
    Env, Flow, List, Original, Screen, Step, check_screen, dispatch_screen,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::player3_details_loop",
    rva(0x477C10),
    thunk,
    &ORIGINAL,
)
.private_copy(0x287)
.relocate_calls(&[
    0x8D, 0x92, 0x9E, 0xA3, 0xF2, 0x101, 0x149, 0x167, 0x1BE, 0x237,
])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn each_frame(g: &mut Regs, env: &mut dyn Env) -> Flow {
    unsafe {
        if game_read::<u32>(rva(0x7556A8)) == 0x1 {
            env.call(rva(0x467C63), g);
            if game_read::<u32>(rva(0x7556A8)) != 0x0 {
                return Flow::NextFrame;
            }
        }
        Flow::On
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74F1CC),
    saved_item: rva(0x7476B4),
    lists: &[
        List {
            pointer: rva(0x7554F4), // g_unreadList
            table: rva(0x758B72),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758CCA),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591F2),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x75901E),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758CCA),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591F2),
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::EaxAddr(rva(0x731D29)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[Step::Write(rva(0x75598C), 0x2)],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
    escape_leaves: false,
    each_frame: &[Step::DoFlow(each_frame)],
    short_action_check: true,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
    before_asking: &[],
    ask_unless: None,
    leaving: &[],
    leave_saved_item: None,
    keys_menu: None,
    after_restored: &[],
    result: None,
    late_highlight: false,
    before_first_frame: &[],
    before_frame: &[],
    after_blink: &[],
    restart_action_check: false,
    always_asks: false,
    also_reads: &[],
    also_writes: &[rva(0x75598C)],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x477C10),
    len: 0x287,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x06, 0x88, 0x75, 0x00, 0x00],
    calls: &[0x8D, 0x9E, 0xA3, 0xF2, 0x101, 0x149, 0x167, 0x1BE, 0x237],
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
