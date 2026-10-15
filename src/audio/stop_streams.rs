//! `audio::stop_streams` (0x40C5E8): both stream commands with eax 0.
//!
//! In:  ebx, ecx, edx = passed on (callers: the thread's end 0x409DD6, the mode change
//!      0x40A43C)
//! Out: every register preserved
//!
//! `audio::stream_command` and then `audio::music_command`, each with eax 0 and the
//! caller's ebx, ecx and edx: with action 0 (or any but 1 and 3) both stop. Shadow mode
//! runs the original alone. Self-test: a redirected copy whose two calls go to stand-ins
//! noting their registers, 4 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::stream::SOUND_STREAM_RVA;
use crate::audio::stream_commands::{MusicVars, game_part, music_command, stream_command};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::stop_streams", rva(0x40C5E8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Its two calls: 0 `stream_command`, 1 `music_command`, with the registers they get.
pub fn stop_streams(
    r: &Regs,
    command: &mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    for which in 0..2 {
        let mut c = Regs { eax: 0, ..*r };
        command(which, &mut c)?;
    }
    Ok(())
}

pub fn game_command(which: u32, r: &mut Regs) -> Result<(), Refused> {
    if which == 0 {
        let player = game_addr(SOUND_STREAM_RVA) as u32;
        stream_command(r.eax, r.ecx as i32, r.edx, player, &mut game_part)
    } else {
        music_command(
            r.eax,
            r.ebx,
            r.ecx as i32,
            r.edx,
            &MusicVars::game(),
            &mut game_part,
        )
    }
}

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    stop_streams(r, &mut game_command)?;
    Ok(Leaves {
        eax: r.eax,
        carry: None,
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

static NOTED: Mutex<Vec<[u32; 5]>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    NOTED
        .lock()
        .unwrap()
        .push([place, r.eax, r.ebx, r.ecx, r.edx]);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40C5E8,
            len: 0x11,
            entry: &[0x60, 0x33, 0xC0, 0xE8, 0xA4],
            calls: &[0x03, 0x0A],
            jumps_out: &[],
        },
        &|callee| {
            [0x40C194, 0x40C4DD]
                .iter()
                .position(|&t| rva(t) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[],
        0,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    for edx in [0u32, 1, 2, 3] {
        let before = Regs {
            edx,
            ..Regs::SENTINEL
        };
        NOTED.lock().unwrap().clear();
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = std::mem::take(&mut *NOTED.lock().unwrap());
        let done = stop_streams(&before, &mut |p, r| {
            handler(p, r);
            Ok(())
        });
        let mine = std::mem::take(&mut *NOTED.lock().unwrap());
        t.check(
            done.is_ok() && ro.preserved_except(&before, 0) && theirs == mine,
            || format!("edx {edx}: original {theirs:x?}; rust {mine:x?}"),
        );
    }
    t.finish()
}
