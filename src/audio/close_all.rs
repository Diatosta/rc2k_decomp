//! `audio::close_all` (0x409DD5): everything of the sound closed (the audio thread's end).
//!
//! In:  ebx, ecx, edx = handed to `audio::stop_streams` (edx its action)
//! Out: every register preserved
//!
//! `audio::stop_streams`, `audio::free_voices`, `audio::close_way`,
//! `audio::release_primary`, `audio::close_sound`, then `g_soundMode` 0. One caller, the
//! audio thread. Shadow mode runs the original alone. Self-test: a redirected copy whose
//! five calls go to stand-ins (the first noting ebx, ecx, edx), 2 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::close_all", rva(0x409DD5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CALLEES: [usize; 5] = [0x40C5E8, 0x40AF4B, 0x40AC58, 0x40A61F, 0x409EA5];

/// Its calls in order, each with the registers it gets.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

pub fn close_all(r: &Regs, mode: u32, part: Part) -> Result<(), Refused> {
    for place in 0..5 {
        let mut c = *r;
        part(place, &mut c)?;
    }
    unsafe { write(mode, 0) };
    let _ = read;
    Ok(())
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::audio::sound_device::*;
    use crate::audio::voices::*;
    unsafe {
        match place {
            0 => crate::audio::stop_streams::stop_streams(
                r,
                &mut crate::audio::stop_streams::game_command,
            )?,
            1 => crate::audio::free_voices::free_voices(
                &World::game(),
                &crate::audio::free_voices::game_vars(),
                &mut crate::audio::free_voices::game_part,
            )?,
            2 => {
                crate::audio::mixer_set_up::close_way(
                    crate::framework::game_read(SOUND_MODE_RVA),
                    &mut crate::audio::restore::game_part,
                )?;
            }
            3 => release_primary(
                game_addr(PRIMARY_RVA) as u32,
                game_addr(VOLUME_KEPT_RVA) as u32,
                &mut crate::audio::sound_device::game_part,
            )?,
            _ => close_sound(
                game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
                game_addr(WINDOW_RVA) as u32,
                &mut crate::audio::sound_device::game_part,
            )?,
        }
    }
    Ok(())
}

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    close_all(
        r,
        game_addr(crate::audio::voices::SOUND_MODE_RVA) as u32,
        &mut game_part,
    )?;
    Ok(Leaves {
        eax: r.eax,
        carry: None,
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

static NOTED: Mutex<Vec<[u32; 4]>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    let regs = if place == 0 {
        [r.ebx, r.ecx, r.edx]
    } else {
        [0; 3]
    };
    NOTED
        .lock()
        .unwrap()
        .push([place, regs[0], regs[1], regs[2]]);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = Box::new([0u32; 1]);
    let at = vars.as_mut_ptr() as u32;
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x409DD5,
            len: 0x26,
            entry: &[0x60, 0xE8, 0x0D, 0x28, 0x00, 0x00],
            calls: &[0x01, 0x06, 0x0B, 0x10, 0x15],
            jumps_out: &[],
        },
        &|callee| {
            CALLEES
                .iter()
                .position(|&c| rva(c) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[(0x4F82BC, at)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    for edx in [1u32, 0] {
        let run = |rust: bool| {
            unsafe { (at as *mut u32).write_volatile(0x2441220) };
            NOTED.lock().unwrap().clear();
            let before = Regs {
                edx,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            let done = if rust {
                close_all(&before, at, &mut |p, r| {
                    handler(p, r);
                    Ok(())
                })
                .is_ok()
            } else {
                unsafe { call_game(&mut ro, c.addr()) };
                ro.preserved_except(&before, 0)
            };
            (done, std::mem::take(&mut *NOTED.lock().unwrap()), unsafe {
                (at as *const u32).read_volatile()
            })
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!("edx {edx}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    drop(vars);
    t.finish()
}
