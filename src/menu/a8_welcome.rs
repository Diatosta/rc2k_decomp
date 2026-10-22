//! `menu::a8_welcome` (0x47F3C7): the A8 championship's welcome: the picture
//! `a8_welc.png` with the title bar (`gfx::load_picture_with_overlay`; not read: fatal
//! error 0x80712345, no message), converted; frames of it (`screens::run_frame` with
//! `menu::picture_frame`) until a key (`input::take_key`). Escape (1) asks about leaving
//! (`menu::confirm_leaving`): a no starts over. Then one more key taken (dropped).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `menu::run_a8_championship` (whose `g_eventMode` 0x13 picks it). The calls
//! through `event_flow::Call` (in the game `game_call`). The loop starts at +1: a private
//! copy (0x48 bytes, its calls and the jump to the fatal stub relocated) and a jump
//! written by hand. Ungated (it waits). Shadow mode runs the original alone. Self-test:
//! `event_flow::test_flow`, keys and answers by script.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::menu::event_flow::*;
use crate::menu::flow_game::Game;
use rc2k_core::menu::welcome::{self, ESCAPE};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::a8_welcome", rva(0x47F3C7), thunk, &ORIGINAL)
    .private_copy(0x48)
    .relocate_calls(&[0xD, 0x12, 0x18, 0x23, 0x28, 0x33, 0x41])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

pub unsafe fn a8_welcome(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::welcome::a8_welcome`.
    welcome::a8_welcome(&mut Game::new(entry, call))?;
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { a8_welcome(r, &mut game_call) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { a8_welcome(r, &mut game_call) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let flow = Flow {
        address: 0x47F3C7,
        len: 0x48,
        // pusha; lea eax, aFilesGGraphics_21
        entry: &[0x60, 0x8D, 0x05, 0x2F, 0x1C, 0x73, 0x00],
        calls: &[0xD, 0x18, 0x23, 0x28, 0x33, 0x41],
        fatal_jumps: &[0x12],
        writes: &[ANSWER],
    };
    let k = Answer::key;
    let scripts: Vec<Script> = vec![
        vec![(TAKE_KEY, vec![k(0x1C)])],
        vec![(TAKE_KEY, vec![k(0), k(0), k(0x39)])],
        vec![
            (TAKE_KEY, vec![k(ESCAPE), k(0), k(ESCAPE)]),
            (
                CONFIRM_LEAVING,
                vec![Answer::write(ANSWER, 0), Answer::yes()],
            ),
        ],
        vec![
            (TAKE_KEY, vec![k(ESCAPE), k(0x1C)]),
            (CONFIRM_LEAVING, vec![Answer::none()]),
        ],
        vec![(LOAD_PICTURE_WITH_OVERLAY, vec![Answer::carry(true)])],
    ];
    let prepare = || set(ANSWER, 0);
    let cases: Vec<Case> = scripts
        .into_iter()
        .enumerate()
        .map(|(n, script)| Case {
            what: format!("script {n}"),
            prepare: &prepare,
            script,
            before: Regs::SENTINEL,
        })
        .collect();
    test_flow(t, &flow, &[], &cases, &|r, c| unsafe { a8_welcome(r, c) })
}
