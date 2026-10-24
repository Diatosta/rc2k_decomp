//! `menu::view_stage_loop` (0x47FD37): the stage details opened from the service area of a
//! championship, by its title "VER ETAPA" and its draw list, which is that of
//! `screens::draw_service_stage_info` (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; it has a step of its own once a
//! frame; the steps that are more than a call or a store are the functions below, generated
//! from the listing.
//! The name is a reading from the title string of its menu, "VER ETAPA", not yet a visit.
//! Callers: `sub_47037E`.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{Callee, Carried, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

// The original pushes `dword_75595C` and `dword_755958` at its entry and pops them at its
// end, with the whole screen in between: the two are kept here across the loop's steps
// (written by hand; the generated functions each have a stack of their own).
thread_local! {
    static KEPT: std::cell::RefCell<Vec<u32>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn take_kept() -> u32 {
    KEPT.with_borrow_mut(|stack| stack.pop().expect("a pop without its push"))
}

unsafe fn before(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        KEPT.with_borrow_mut(|stack| stack.push(game_read(rva(0x75595C))));
        KEPT.with_borrow_mut(|stack| stack.push(game_read(rva(0x755958))));
        c.arg = game_read(rva(0x75595C));
        game_write(rva(0x755964), c.arg);
        c.arg = game_read(rva(0x755958));
        game_write(rva(0x755960), c.arg);
    }
    Ok(())
}

unsafe fn after_lists(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        Callee::CountStagePages.run(c)?;
        Callee::StyleChosenStageItem.run(c)?;
        if game_read::<u32>(rva(0x755968)) == 0x0 {
            Callee::StyleStageItem.run(c)?;
        }
        game_write(rva(0x755700), 0x3u32);
    }
    Ok(())
}

unsafe fn after(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        game_write(rva(0x755958), take_kept());
        game_write(rva(0x75595C), take_kept());
        Callee::StopStream.run(c)?;
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74DAA0),
    saved_item: rva(0x74776C),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x759096),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758D86),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x75925A),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[Step::Do(after_lists)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after_first_frame: &[Step::Call(Callee::PlayStageSpeech)],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Do(after)],
    escape_leaves: false,
    each_frame: &[Step::Call(Callee::StepCarStream)],
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
};
