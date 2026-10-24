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
//! `sub_4AA040`, `sub_4AA040`. Rust callers run its `entry` (`routes.rs`); the registers above are
//! the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::menu::screen_loop::{
    Callee, Carried, Flow, List, Screen, Step, carousel, prepare_scene, quitting, read_at,
};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        // The current player's +0x30 (its car).
        let current: u32 = game_read(rva(0x75598C));
        let player =
            read_at((game_addr(rva(0x75647C)) as u32).wrapping_add(current.wrapping_mul(4)));
        c.arg = read_at(player.wrapping_add(0x30));
        game_write(rva(0x7554F0), c.arg);
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0xDu32);
        carousel(c)?;
        c.arg = game_read(rva(0x7476DC));
        game_write(rva(0x755525), c.arg);
        c.arg = c.arg.wrapping_sub(1);
        game_write(rva(0x75AA7C), c.arg);
        game_write(rva(0x7554EC), 0x0u32);
        game_write(rva(0x75AA7C), 0x0u32);
        game_write(rva(0x755990), 0x0u32);
        crate::menu::car_choice::find_player_car()?;
        game_write(rva(0x623FB0), 0x7u32);
        game_write(rva(0x623FB4), 0xFFFFFFCEu32);
        game_write(rva(0x624080), 0x100u32);
        prepare_scene(c);
    }
    Ok(())
}

unsafe fn after_first_frame(_c: &mut Carried) -> Result<Flow, Refused> {
    if quitting() as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

unsafe fn leaving(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(rva(0x758806)) == 0x1 {
            Callee::GoBack.run(c)?;
        }
    }
    Ok(())
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
        Step::ArgAddr(rva(0x731F19)),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
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
