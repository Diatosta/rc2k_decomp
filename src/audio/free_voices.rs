//! `audio::free_voices` (0x40AF4B): command 4: every sound's voices and buffers given back.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! If voices are in use (`g_voiceCount`): `audio::release_eax`, `audio::release_listener`,
//! `audio::release_voice` on each of them, `g_voiceCount` 0. Then every sample record's
//! `first_voice` 0, `audio::clear_channels`, `g_soundBufferLost`, `g_commit3d`,
//! `g_channelsUsed` and both lists 0, and the samples' memory given back (`sub_409E0F`,
//! `memory::free_list` on `g_sampleMemory`, inlined). Callers: `sub_409DD5` (the thread's
//! end), `audio::set_mode`, `audio::take_commands`.
//! Refuses more than 0x200 voices in use.
//! Shadow mode runs the original alone. Self-test: a redirected copy whose variables are
//! the test's and whose five calls note themselves, 8 cases.

use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::free_voices", rva(0x40AF4B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// What it calls: 0 `release_eax`, 1 `release_listener`, 2 `release_voice` (the voice),
/// 3 `clear_channels`, 4 the samples' memory.
pub type Part<'a> = &'a mut dyn FnMut(u32, u32) -> Result<(), Refused>;

/// Where its variables are: `World` and these.
#[derive(Clone, Copy)]
pub struct Vars {
    pub voice_count: u32,
    pub voices: u32,
    pub lost: u32,
    pub commit: u32,
}

pub unsafe fn free_voices(w: &World, v: &Vars, part: Part) -> Result<(), Refused> {
    unsafe {
        let count = read(v.voice_count);
        if count > VOICES {
            return Err(BadInput("more than 0x200 voices").into());
        }
        if count != 0 {
            part(0, 0)?;
            part(1, 0)?;
            for n in 0..count {
                part(2, v.voices + n * std::mem::size_of::<Voice>() as u32)?;
            }
            write(v.voice_count, 0);
        }
        for n in 0..SAMPLES {
            sample(w, n).first_voice = 0;
        }
        part(3, 0)?;
        write(v.lost, 0);
        write(v.commit, 0);
        write(w.channels_used, 0);
        write(w.playing, 0);
        write(w.stopping, 0);
        part(4, 0)
    }
}

fn game_vars() -> Vars {
    Vars {
        voice_count: game_addr(VOICE_COUNT_RVA) as u32,
        voices: game_addr(VOICES_RVA) as u32,
        lost: game_addr(BUFFER_LOST_RVA) as u32,
        commit: game_addr(COMMIT_RVA) as u32,
    }
}

fn game_part(which: u32, at: u32) -> Result<(), Refused> {
    unsafe {
        match which {
            0 => crate::audio::release_eax::release_eax(crate::audio::release_eax::game_vars()),
            1 => crate::audio::release_listener::release_listener(game_addr(LISTENER_RVA) as u32),
            2 => crate::audio::release_voice::release_voice_in_game(at),
            3 => crate::audio::clear_channels::clear_channels(World::game().channels),
            _ => crate::memory::free_list::free_list(game_addr(SAMPLE_MEMORY_RVA) as *mut _)?,
        }
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { free_voices(&World::game(), &game_vars(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

static NOTED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    NOTED
        .lock()
        .unwrap()
        .push((place, if place == 2 { r.esi } else { 0 }));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The records, six voices, the nine variables.
    let records = SAMPLES as usize * 0x10;
    let words = records + 6 * 0x1C + 9;
    let mut block = vec![0u32; words];
    let base = block.as_mut_ptr() as u32;
    let voices = base + records as u32 * 4;
    let vars = voices + 6 * 0x70;
    let w = World {
        samples: base,
        playing: vars,
        stopping: vars + 4,
        channels_used: vars + 8,
        channel_count: vars + 12,
        channels: vars + 16,
        sound_mode: vars + 20,
    };
    let v = Vars {
        voice_count: vars + 24,
        voices,
        lost: vars + 28,
        commit: vars + 32,
    };
    let calls = [0xA, 0xF, 0x20, 0x4D, 0x84];
    let targets = [0x40D2F0, 0x40C74C, 0x40B767, 0x40ACA1, 0x409E0F];
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40AF4B,
            len: 0x8B,
            entry: &[0x60, 0x83, 0x3D, 0x48, 0x83, 0x4F, 0x00, 0x00],
            calls: &calls,
            jumps_out: &[],
        },
        &|callee| {
            targets
                .iter()
                .position(|&t| rva(t) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[
            (0x4F8340, w.playing),
            (0x4F8344, w.stopping),
            (0x506350, w.channels_used),
            (0x4F8348, v.voice_count),
            (0x4F834C, v.voices),
            (0x50A7A4, w.samples),
            (0x4F82CC, v.lost),
            (0x4F82D0, v.commit),
        ],
        10,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    for count in [0u32, 1, 2, 6, 0, 3, 5, 4] {
        let fill = |b: &mut Vec<u32>| {
            for (i, x) in b.iter_mut().enumerate() {
                *x = 0xC000_0000 ^ (i as u32).wrapping_mul(count + 7);
            }
            unsafe { write(v.voice_count, count) };
        };
        fill(&mut block);
        NOTED.lock().unwrap().clear();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (block.clone(), std::mem::take(&mut *NOTED.lock().unwrap()));
        fill(&mut block);
        let done = unsafe {
            free_voices(&w, &v, &mut |which, at| {
                NOTED.lock().unwrap().push((which, at));
                Ok(())
            })
        };
        let mine = (block.clone(), std::mem::take(&mut *NOTED.lock().unwrap()));
        let differ = (0..words).find(|&i| theirs.0[i] != mine.0[i]);
        t.check(
            done.is_ok() && differ.is_none() && theirs.1 == mine.1
                && ro.preserved_except(&Regs::SENTINEL, 0),
            || {
                format!(
                    "{count} voices: first difference at word {differ:x?}; calls original {:x?}, rust {:x?} {done:?}",
                    theirs.1, mine.1
                )
            },
        );
    }
    t.finish()
}
