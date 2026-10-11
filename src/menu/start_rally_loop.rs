//! `menu::start_rally_loop` (0x475C12): the mode menu, titled "COMEÇAR RALI".
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `sral.png`; leaving is not asked about; the
//! next frame begins with the steps after the first frame; it has steps of its own first on
//! leaving; the quit switch is not looked at; the steps that are more than a call or a store
//! are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "COMEÇAR RALI", not yet a visit.
//! Callers: none in code: it is reached through a table. Ungated thunk; shadow mode runs the
//! original alone; each real call writes a line to hook.log. Self-test:
//! `screen_loop::check_screen`. Manual patch with a private copy (0x308 bytes, 12 calls and
//! jumps relocated): a call or a jump target sits in the first bytes.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::screen_loop::{
    Env, Flow, List, Original, Screen, Step, check_screen, dispatch_screen,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::start_rally_loop", rva(0x475C12), thunk, &ORIGINAL)
    .private_copy(0x308)
    .relocate_calls(&[
        0x15, 0x48, 0xE3, 0xE8, 0xF4, 0xF9, 0x148, 0x165, 0x1AA, 0x208, 0x2AA, 0x2CF,
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

unsafe fn set_up_first(g: &mut Regs, env: &mut dyn Env) {
    unsafe {
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0x1u32);
        env.call(rva(0x4904DD), g);
        g.eax = game_read(rva(0x7476C4));
        game_write(rva(0x755525), g.eax);
        g.eax = g.eax.wrapping_add(0xFFFFFFFF);
        game_write(rva(0x75AA7C), g.eax);
        game_write(rva(0x623FB0), 0x7u32);
        game_write(rva(0x623FB4), 0xAu32);
        game_write(rva(0x624080), 0x100u32);
        env.call(rva(0x46F7FD), g);
    }
}

unsafe fn after_first_frame(_g: &mut Regs, _env: &mut dyn Env) -> Flow {
    unsafe {
        if game_read::<u32>(rva(0x758806)) == 0x1 {
            game_write(rva(0x758806), 0x0u32);
            return Flow::SetUp;
        }
        Flow::On
    }
}

unsafe fn leaving(g: &mut Regs, env: &mut dyn Env) {
    unsafe {
        if game_read::<u32>(rva(0x758806)) == 0x1 {
            env.call(rva(0x471C82), g);
        }
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74E2E0),
    saved_item: rva(0x7476C4),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758EEE),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758B82),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x75912A),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::Write(rva(0x7558D8), 0x1),
        Step::EaxAddr(rva(0x731C9F)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: true,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: true,
    before_asking: &[],
    ask_unless: None,
    leaving: &[Step::Do(leaving)],
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
    also_writes: &[
        rva(0x623FB0),
        rva(0x623FB4),
        rva(0x624080),
        rva(0x7554EC),
        rva(0x7558D8),
        rva(0x75AA74),
        rva(0x75AA7C),
    ],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x475C12),
    len: 0x308,
    entry_bytes: &[0x60, 0xC7, 0x05, 0xEC, 0x54, 0x75, 0x00, 0x01],
    calls: &[
        0x15, 0x48, 0xE3, 0xF4, 0xF9, 0x148, 0x165, 0x1AA, 0x208, 0x2AA, 0x2CF,
    ],
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
