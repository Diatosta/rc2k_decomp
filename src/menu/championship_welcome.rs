//! `menu::championship_welcome` (0x47F314): the championship's welcome: unless a saved
//! game goes on (`dword_755970` 1), the speech `;3files\%speech\rally\rally0.wav` started
//! (`sound::start_stream`, ecx `dword_755698`) and the commands waited for;
//! `dword_75555D`, `dword_755561` and `g_yesNoAnswer` 0; the picture `mob_welc.png` with
//! the title bar (`gfx::load_picture_with_overlay`; not read: fatal error 0x80712345, no
//! message), converted; a key taken (dropped); one frame of it (`screens::run_frame`
//! with `menu::picture_frame`); then three seconds for a key (`system::wait_or_key`), and
//! without one, keys taken until one comes while the speech plays (`sound::stream_playing`)
//! and the game is not quitting (`g_quitting`): the speech's end or the quitting end it
//! there. A key other than Escape (1) stops the speech and ends it; Escape stops it and
//! asks about leaving (`menu::confirm_leaving`): a yes stops it again and ends, a no
//! starts over.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `menu::run_championship` (`g_eventMode` 2). The calls through
//! `event_flow::Call` (in the game `game_call`: the Rust versions, the sound commands and
//! the frame through their hooks). Ungated (it waits). Shadow mode runs the original
//! alone. Self-test: `event_flow::test_flow`, keys and the speech by script.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::menu::event_flow::*;
use crate::menu::flow_game::Game;
use rc2k_core::menu::welcome::{self, ESCAPE};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::championship_welcome",
    rva(0x47F314),
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

pub fn addr(a: usize) -> u32 {
    game_addr(rva(a)) as u32
}

pub fn zero(g: &Regs) -> bool {
    g.eflags & 0x40 != 0
}

pub unsafe fn championship_welcome(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::welcome::championship_welcome`.
    welcome::championship_welcome(&mut Game::new(entry, call))?;
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { championship_welcome(r, &mut game_call) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { championship_welcome(r, &mut game_call) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

const CALLS: [usize; 12] = [
    0x1A, 0x1F, 0x4E, 0x59, 0x5E, 0x69, 0x73, 0x7A, 0x8A, 0x95, 0x9A, 0xAC,
];

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let flow = Flow {
        address: 0x47F314,
        len: 0xB3,
        // pusha; cmp dword_755970, 1
        entry: &[0x60, 0x83, 0x3D, 0x70, 0x59, 0x75, 0x00, 0x01],
        calls: &CALLS,
        fatal_jumps: &[0x53],
        writes: &[LEAVE, LEAVE_561, ANSWER, CONTINUING],
    };
    let k = Answer::key;
    // (g_quitting is left as it is: the audio thread reads it.)
    if get(QUITTING) != 0 {
        return t.skip("the game is quitting");
    }
    let scripts: Vec<(u32, Script)> = vec![
        (1, vec![]),
        (0, vec![(WAIT_OR_KEY, vec![k(0x1C)])]),
        (0, vec![]),
        (
            0,
            vec![
                (STREAM_PLAYING, vec![Answer::zero(true), Answer::zero(true)]),
                (TAKE_KEY, vec![k(0), k(0), k(5)]),
            ],
        ),
        (
            0,
            vec![
                (WAIT_OR_KEY, vec![k(ESCAPE), k(ESCAPE)]),
                (CONFIRM_LEAVING, vec![Answer::none(), Answer::yes()]),
            ],
        ),
        (
            0,
            vec![
                (STREAM_PLAYING, vec![Answer::zero(true)]),
                (TAKE_KEY, vec![k(0), k(ESCAPE)]),
                (CONFIRM_LEAVING, vec![Answer::yes()]),
            ],
        ),
        (
            0,
            vec![(LOAD_PICTURE_WITH_OVERLAY, vec![Answer::carry(true)])],
        ),
    ];
    let prepares: Vec<Box<dyn Fn()>> = scripts
        .iter()
        .map(|&(continuing, _)| Box::new(move || set(CONTINUING, continuing)) as Box<dyn Fn()>)
        .collect();
    let cases: Vec<Case> = scripts
        .into_iter()
        .zip(&prepares)
        .enumerate()
        .map(|(n, ((c, script), prepare))| Case {
            what: format!("script {n}, continuing {c}"),
            prepare: prepare.as_ref(),
            script,
            before: Regs::SENTINEL,
        })
        .collect();
    test_flow(t, &flow, &[], &cases, &|r, c| unsafe {
        championship_welcome(r, c)
    })
}
