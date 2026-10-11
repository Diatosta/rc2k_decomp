//! `menu::information_loop` (0x48E169): the loop of the information screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `info.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "INFORMAÇÃO", not yet a visit.
//! Callers: none in code: it is reached through a table. Ungated thunk; shadow mode runs the
//! original alone; each real call writes a line to hook.log. Self-test:
//! `screen_loop::check_screen`. Manual patch with a private copy (0x32B bytes, 13 calls and
//! jumps relocated): a call or a jump target sits in the first bytes.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::screen_loop::{
    Env, List, Original, Screen, Step, check_screen, dispatch_screen, popa, pusha,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::information_loop", rva(0x48E169), thunk, &ORIGINAL)
    .private_copy(0x32B)
    .relocate_calls(&[
        0x8, 0x2C, 0x71, 0x11C, 0x121, 0x12D, 0x132, 0x181, 0x186, 0x1CE, 0x22C, 0x2CE, 0x323,
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
        let mut all = Vec::new();
        pusha(g, &mut all);
        g.eax = game_addr(rva(0x4DEAD0)) as u32;
        env.call(rva(0x40142B), g);
        popa(g, &mut all);
        game_write(rva(0x623FB0), 0xFFFFFF64u32);
        game_write(rva(0x623FB4), 0x27u32);
        game_write(rva(0x624080), 0x140u32);
        env.call(rva(0x46F7FD), g);
        game_write(rva(0x6241DC), 0xFFFFFFD8u32);
        game_write(rva(0x6241E0), 0x2F8u32);
        game_write(rva(0x6241EC), game_read::<i32>(rva(0x6241DC)) as f32);
        game_write(rva(0x6241F0), game_read::<i32>(rva(0x6241E0)) as f32);
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0x9u32);
        env.call(rva(0x4904DD), g);
        g.eax = game_read(rva(0x7476BC));
        game_write(rva(0x755525), g.eax);
        g.eax = g.eax.wrapping_add(0xFFFFFFFF);
        game_write(rva(0x75AA7C), g.eax);
    }
}

unsafe fn after(g: &mut Regs, env: &mut dyn Env) {
    let mut all = Vec::new();
    pusha(g, &mut all);
    g.eax = game_addr(rva(0x4DEADF)) as u32;
    env.call(rva(0x40142B), g);
    popa(g, &mut all);
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74F90C),
    saved_item: rva(0x7476BC),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758FEE),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758C82),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591CA),
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
        Step::EaxAddr(rva(0x731DB4)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Do(after)],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
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
    also_writes: &[
        rva(0x623FB0),
        rva(0x623FB4),
        rva(0x624080),
        rva(0x6241DC),
        rva(0x6241E0),
        rva(0x6241EC),
        rva(0x6241F0),
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
    function: rva(0x48E169),
    len: 0x32B,
    entry_bytes: &[0x60, 0x60, 0x8D, 0x05, 0xD0, 0xEA, 0x4D, 0x00],
    calls: &[
        0x8, 0x2C, 0x71, 0x11C, 0x12D, 0x132, 0x181, 0x186, 0x1CE, 0x22C, 0x2CE, 0x323,
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
