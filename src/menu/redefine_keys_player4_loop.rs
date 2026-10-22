//! `menu::redefine_keys_player4_loop` (0x479F52): the keyboard assignment screen "REDEFINIR
//! TECLADO" of the fourth car.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `defkeys.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; it has a step of its own once
//! a frame; the steps that are more than a call or a store are the functions below, generated
//! from the listing.
//! The name is a reading from the title string of its menu, "REDEFINIR TECLADO", not yet a
//! visit.
//! Callers: none in code: it is reached through a table.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::screen_loop::{
    Env, Flow, List, Screen, Step, copy_bytes, popa, pusha, read_at, screen_entry,
};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn before(g: &mut Regs, _env: &mut dyn Env) {
    unsafe {
        let mut all = Vec::new();
        let mut stack: Vec<u32> = Vec::new();
        stack.push(g.eax);
        g.eax = 0x3;
        g.edi = read_at((game_addr(rva(0x75647C)) as u32).wrapping_add(g.eax.wrapping_mul(4)));
        g.eax = stack.pop().expect("a pop without its push");
        g.edi = g.edi.wrapping_add(0x1C);
        pusha(g, &mut all);
        g.ecx = 0xC;
        g.esi = g.edi;
        g.edi = game_addr(rva(0x755595)) as u32;
        copy_bytes(g);
        popa(g, &mut all);
        game_write(rva(0x75598C), 0x3u32);
    }
}

unsafe fn after_first_frame(_g: &mut Regs, env: &mut dyn Env) -> Flow {
    if env.quitting() as u32 == 0x1 {
        return Flow::Leave;
    }
    Flow::On
}

unsafe fn each_frame(_g: &mut Regs, _env: &mut dyn Env) -> Flow {
    unsafe {
        if game_read::<u32>(rva(0x7556AC)) == 0x1 && game_read::<u32>(rva(0x7556AC)) != 0x0 {
            return Flow::NextFrame;
        }
        Flow::On
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74C868),
    saved_item: rva(0x747754),
    lists: &[
        List {
            pointer: rva(0x7554F4), // g_unreadList
            table: rva(0x758B72),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758E06),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759292),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x7590CE),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758E06),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759292),
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::EaxAddr(rva(0x73236A)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
    escape_leaves: false,
    each_frame: &[Step::DoFlow(each_frame)],
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
    also_writes: &[rva(0x755595), rva(0x755599), rva(0x75559D), rva(0x75598C)],
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    screen_entry("menu::redefine_keys_player4_loop", r, &SCREEN)
}
