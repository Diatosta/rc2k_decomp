//! `menu::select_car_loop` (0x47AC0C): the car selection screen, which the starters of
//! every kind of event call.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `cspecs.png`; leaving is not asked about; it
//! has steps of its own first on leaving; the steps that are more than a call or a store are
//! the functions below, generated from the listing.
//! Screen: the car selection when an event is started (the owner called it the car select
//! menu, 2026-10-12; it ran in the Rust version under each of the five start screens that
//! day). Named `car_specs_47ac0c_loop` until then.
//! Callers: `sub_4182FA`, `sub_482382`, `sub_4824C1`, `sub_4824C1`, `sub_4826A2`, `sub_4826A2`,
//! `sub_4828B0`, `sub_4828B0`, `sub_4828B0`, `sub_482B7A`, `sub_482B7A`, `sub_482B7A`,
//! `sub_482B7A`, `sub_487E2C`, `sub_48AE39`, `sub_48AFD0`, `sub_48AFD0`, `sub_48B23D`,
//! `sub_48B23D`, `sub_48B54B`, `sub_48B54B`, `sub_48B54B`, `sub_48B90F`, `sub_48B90F`,
//! `sub_48B90F`, `sub_48B90F`, `sub_4A25F4`, `sub_4A26B6`, `sub_4A26B6`, `sub_4A27B5`,
//! `sub_4A27B5`, `sub_4A28B4`, `sub_4A28B4`, `sub_4A28B4`, `sub_4A29F4`, `sub_4A29F4`,
//! `sub_4A29F4`, `sub_4A29F4`, `sub_4A9C9B`, `sub_4A9D5C`, `sub_4A9D5C`, `sub_4A9E3D`,
//! `sub_4A9E3D`, `sub_4A9F1E`, `sub_4A9F1E`, `sub_4A9F1E`, `sub_4AA040`, `sub_4AA040`,
//! `sub_4AA040`, `sub_4AA040`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::screen_loop::{Env, Flow, List, Screen, Step, read_at, screen_entry};

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
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    screen_entry("menu::select_car_loop", r, &SCREEN)
}
