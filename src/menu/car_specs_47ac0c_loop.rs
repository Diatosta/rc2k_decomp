//! `menu::car_specs_47ac0c_loop` (0x47AC0C): a screen over the car specification background
//! that the starters of every kind of event call.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `cspecs.png`; leaving is not asked about; it
//! has steps of its own first on leaving; the steps that are more than a call or a store are
//! the functions below, generated from the listing.
//! The name is a reading from its background and callers, not yet a visit.
//! Callers: `sub_4182FA`, `sub_482382`, `sub_4824C1`, `sub_4824C1`, `sub_4826A2`, `sub_4826A2`,
//! `sub_4828B0`, `sub_4828B0`, `sub_4828B0`, `sub_482B7A`, `sub_482B7A`, `sub_482B7A`,
//! `sub_482B7A`, `sub_487E2C`, `sub_48AE39`, `sub_48AFD0`, `sub_48AFD0`, `sub_48B23D`,
//! `sub_48B23D`, `sub_48B54B`, `sub_48B54B`, `sub_48B54B`, `sub_48B90F`, `sub_48B90F`,
//! `sub_48B90F`, `sub_48B90F`, `sub_4A25F4`, `sub_4A26B6`, `sub_4A26B6`, `sub_4A27B5`,
//! `sub_4A27B5`, `sub_4A28B4`, `sub_4A28B4`, `sub_4A28B4`, `sub_4A29F4`, `sub_4A29F4`,
//! `sub_4A29F4`, `sub_4A29F4`, `sub_4A9C9B`, `sub_4A9D5C`, `sub_4A9D5C`, `sub_4A9E3D`,
//! `sub_4A9E3D`, `sub_4A9F1E`, `sub_4A9F1E`, `sub_4A9F1E`, `sub_4AA040`, `sub_4AA040`,
//! `sub_4AA040`, `sub_4AA040`. Ungated thunk; shadow mode runs the original alone; each real
//! call writes a line to hook.log. Self-test: `screen_loop::check_screen`. Manual patch with a
//! private copy (0x340 bytes, 14 calls and jumps relocated): a call or a jump target sits in
//! the first bytes.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::screen_loop::{
    Env, Flow, List, Original, Screen, Step, check_screen, dispatch_screen, read_at,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::car_specs_47ac0c_loop",
    rva(0x47AC0C),
    thunk,
    &ORIGINAL,
)
.private_copy(0x340)
.relocate_calls(&[
    0x2B, 0x5E, 0x81, 0xF2, 0x123, 0x128, 0x134, 0x139, 0x188, 0x19A, 0x1E2, 0x240, 0x2E2, 0x307,
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
        let mut stack: Vec<u32> = Vec::new();
        stack.push(g.eax);
        g.eax = game_read(rva(0x75598C));
        g.edi = read_at((game_addr(rva(0x75647C)) as u32).wrapping_add(g.eax.wrapping_mul(4)));
        g.eax = stack.pop().expect("a pop without its push");
        g.eax = read_at(g.edi.wrapping_add(0x30));
        game_write(rva(0x7554F0), g.eax);
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0xDu32);
        env.call(rva(0x4904DD), g);
        g.eax = game_read(rva(0x7476DC));
        game_write(rva(0x755525), g.eax);
        g.eax = g.eax.wrapping_add(0xFFFFFFFF);
        game_write(rva(0x75AA7C), g.eax);
        game_write(rva(0x7554EC), 0x0u32);
        game_write(rva(0x75AA7C), 0x0u32);
        game_write(rva(0x755990), 0x0u32);
        env.call(rva(0x4708FD), g);
        game_write(rva(0x623FB0), 0x7u32);
        game_write(rva(0x623FB4), 0xFFFFFFCEu32);
        game_write(rva(0x624080), 0x100u32);
        env.call(rva(0x46F7FD), g);
    }
}

unsafe fn after_first_frame(_g: &mut Regs, env: &mut dyn Env) -> Flow {
    if env.quitting() as u32 == 0x1 {
        return Flow::Leave;
    }
    Flow::On
}

unsafe fn leaving(g: &mut Regs, env: &mut dyn Env) {
    unsafe {
        if game_read::<u32>(rva(0x758806)) == 0x1 {
            env.call(rva(0x471C82), g);
        }
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74B918),
    saved_item: rva(0x7476DC),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758F5E),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758C32),
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x7593FE),
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591A2),
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
        Step::EaxAddr(rva(0x731F19)),
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
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
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
        rva(0x7554F0),
        rva(0x755990),
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
    function: rva(0x47AC0C),
    len: 0x340,
    entry_bytes: &[0x60, 0x50, 0xA1, 0x8C, 0x59, 0x75, 0x00, 0x8B],
    calls: &[
        0x2B, 0x5E, 0x81, 0xF2, 0x123, 0x134, 0x139, 0x188, 0x19A, 0x1E2, 0x240, 0x2E2, 0x307,
    ],
};

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The screen reads through this table's entries: without them the original would trap.
    if (0..4).any(|n| unsafe { game_read::<u32>(rva(0x75647C) + n * 4) } == 0) {
        return t.skip("a car object is missing");
    }
    if let Err(why) = check_screen(&mut t, &SCREEN, &THE_ORIGINAL) {
        return t.skip(why);
    }
    t.finish()
}
