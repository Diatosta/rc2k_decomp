//! `audio::start_voice` (0x40A9D1): command 1 on the audio thread: a sound started,
//! changed or stopped.
//!
//! In:  eax = the sound's number (the sender's ax, sign-extended), esi = the command's
//!      record (`sound/queue.rs`: +4 eax, +8 volume, +0xC pan, +0x10 frequency, +0x14 ..
//!      +0x1C the place)
//! Out: every register preserved
//!
//! See `audio/voices.rs`. Nothing for a negative number, a sound not loaded, or one whose
//! first voice is not. With a volume and a voice of kind 0 (or, for another kind or no
//! volume, a voice not playing, unless the sender's eax has bit 31): the first of its
//! copies that is free gets a channel, a free one while `g_channelsUsed` is under
//! `g_channelCount` (then counted), else the channel of the playing voice of the lowest
//! priority below its own (signed; none: nothing), which goes to the stopping list; the
//! voice goes to the front of the playing list, state 1, and its channel names it. A
//! playing voice with no volume is stopped: its channel freed and counted down, the voice
//! to the stopping list. Then, for a voice started or playing: the volume (`DEFAULT`: the
//! record's; held to 0x40, unsigned), the pan (its low byte, signed, held to -0x40 ..
//! 0x40; `pan_sign` -1 if bit 7 of its two low bytes differ), the frequency (also
//! `OWN_FREQUENCY`; held to 100 .. 200,000, signed) and, in 3D, the place's three floats,
//! each marked dirty if it changed; kind 3 marks a restart; and the play bit is set.
//! Refuses, before anything is written: a number of 0x1CD or more (its own or the chosen
//! voice's); no channels left and no voice playing; a playing list that does not end
//! within 0x200 voices; a playing voice to stop without its channel.
//! Compared in shadow mode. Self-test: a redirected copy on a world of the test's, 600
//! states and commands by lot.

use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{Region, dispatch_state};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::start_voice", rva(0x40A9D1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x269;
/// The whole instructions the hook's jump covers: pusha, test eax, eax, js.
const ENTRY_BYTES: [u8; 9] = [0x60, 0x85, 0xC0, 0x0F, 0x88, 0x5E, 0x02, 0x00, 0x00];

// ---- Rust version ----

/// Converted: `rc2k_core::audio::voices::start_voice`; nothing is written when it refuses.
pub unsafe fn start_voice(w: &World, sound: i32, cmd: &[u32; 8]) -> Result<(), Refused> {
    unsafe {
        let mut l = lists(w);
        let mode = read(w.sound_mode);
        rc2k_core::audio::voices::start_voice(&mut GamePool::new(w), &mut l, mode, sound, cmd)?;
        store_lists(w, &l);
    }
    Ok(())
}

// ---- hook ----

pub fn command(at: u32) -> [u32; 8] {
    unsafe { (at as *const [u32; 8]).read_unaligned() }
}

fn rust_on(w: World) -> impl Fn(&Regs) -> Result<Left, Refused> {
    move |r| {
        unsafe { start_voice(&w, r.eax as i32, &command(r.esi)) }?;
        Ok(Left::nothing())
    }
}

/// What a call may write in the game's world: the voices, the channels, the lists.
fn regions() -> Vec<Region> {
    let w = World::game();
    let count = unsafe { read(w.channel_count) }.min(0x100) as usize;
    vec![
        (
            game_addr(VOICES_RVA) as u32,
            VOICES as usize * std::mem::size_of::<Voice>(),
        ),
        (w.channels, count * std::mem::size_of::<MixerChannel>()),
        (w.playing, 8),
        (w.channels_used, 4),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust_on(World::game()));
}

// ---- self-test ----

/// A world of the test's: sample records, voices, channels and the variables, in one block
/// that does not move.
struct TestWorld {
    block: Vec<u32>,
}

const T_SAMPLES: usize = 0;
const T_VOICES: usize = T_SAMPLES + SAMPLES as usize * 0x10;
const T_VOICE_COUNT: usize = 24;
const T_CHANNELS: usize = T_VOICES + T_VOICE_COUNT * 0x1C;
const T_CHANNEL_COUNT: usize = 6;
const T_VARIABLES: usize = T_CHANNELS + T_CHANNEL_COUNT * 0x16;
const T_COMMAND: usize = T_VARIABLES + 8;
const T_LEN: usize = T_COMMAND + 8;
/// The sounds the test uses.
const T_SOUNDS: [u32; 6] = [0, 1, 5, 0x100, 0x1CC, 7];

impl TestWorld {
    fn new() -> Self {
        Self {
            block: vec![0; T_LEN],
        }
    }
    fn at(&self, word: usize) -> u32 {
        self.block.as_ptr() as u32 + word as u32 * 4
    }
    fn world(&self) -> World {
        World {
            samples: self.at(T_SAMPLES),
            playing: self.at(T_VARIABLES),
            stopping: self.at(T_VARIABLES + 1),
            channels_used: self.at(T_VARIABLES + 2),
            channel_count: self.at(T_VARIABLES + 3),
            channels: self.at(T_CHANNELS),
            sound_mode: self.at(T_VARIABLES + 4),
        }
    }
    fn voice(&self, n: usize) -> u32 {
        self.at(T_VOICES + n * 0x1C)
    }
    fn channel(&self, n: usize) -> u32 {
        self.at(T_CHANNELS + n * 0x16)
    }
    /// A state by lot: the sounds' voices in chains, some playing on channels.
    fn fill(&mut self, rng: &mut u32) {
        self.block.fill(0);
        let w = self.world();
        let mut next = || {
            *rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            *rng >> 8
        };
        let channels = (next() % (T_CHANNEL_COUNT as u32 + 1)).max(1);
        let mut voices = 0usize;
        let mut playing = Vec::new();
        unsafe {
            write(w.channel_count, channels);
            write(w.sound_mode, if next() % 2 == 0 { MODE_3D } else { 0 });
            for &s in &T_SOUNDS {
                let r = sample(&w, s);
                r.number = s;
                r.volume = [0x20, 0x40, 0x50, DEFAULT][next() as usize % 4];
                r.pan = next() & 0xFFFF;
                r.frequency = [22050, 50, 300_000, OWN_FREQUENCY][next() as usize % 4];
                r.place = [next(), next(), DEFAULT_PLACE];
                if next() % 5 == 0 {
                    continue;
                }
                let copies = 1 + next() as usize % 3;
                let mut previous = 0;
                for c in 0..copies {
                    if voices == T_VOICE_COUNT {
                        break;
                    }
                    let at = self.voice(voices);
                    voices += 1;
                    let v = voice(at);
                    v.flags = if next() % 8 == 0 { 1 } else { 1 | LOADED };
                    v.sound = s;
                    v.kind = [0, 0, 1, 3][next() as usize % 4];
                    v.priority = (next() % 5) as i32 - 1;
                    v.own_frequency = 11025;
                    v.volume = next() % 0x41;
                    v.pan = (next() % 0x81) as i32 - 0x40;
                    v.frequency = [22050, 11025, 100][next() as usize % 3];
                    v.place = [next(), 0, 0];
                    v.dirty = next() % 0x40;
                    if c == 0 {
                        r.first_voice = at;
                    } else {
                        voice(previous).next_copy = at;
                    }
                    previous = at;
                    if next() % 2 == 0 && playing.len() < channels as usize {
                        playing.push(at);
                    } else {
                        v.state = if next() % 6 == 0 { 2 } else { 0 };
                    }
                }
            }
            // The playing voices, each on a channel, in a list.
            let mut used = 0;
            for (i, &at) in playing.iter().enumerate() {
                let v = voice(at);
                v.state = 1 + next() % 2;
                let c = self.channel(i);
                write(c, 1);
                channel(c).voice = at;
                v.channel = c;
                v.next = playing.get(i + 1).copied().unwrap_or(0);
                used += 1;
            }
            write(w.playing, playing.first().copied().unwrap_or(0));
            write(w.channels_used, used);
            // The stopping list's head is only ever stored: any value does.
            write(
                w.stopping,
                if voices > 0 && next() % 2 == 0 {
                    0x1234_5678
                } else {
                    0
                },
            );
            // The command.
            let cmd = self.at(T_COMMAND) as *mut [u32; 8];
            let sound = T_SOUNDS[next() as usize % T_SOUNDS.len()];
            let bit31 = if next() % 4 == 0 { 0x8000_0000 } else { 0 };
            cmd.write([
                1,
                sound | bit31 | (next() & 0x7F_0000),
                [0, 0x30, 0x41, DEFAULT, 0xFFFF_FF00][next() as usize % 5],
                [DEFAULT, next() & 0xFFFF, 0x80, 0x0140][next() as usize % 4],
                [DEFAULT, OWN_FREQUENCY, 44100, 10, 0x8000_0000][next() as usize % 5],
                [DEFAULT_PLACE, next()][next() as usize % 2],
                [DEFAULT_PLACE, next()][next() as usize % 2],
                [DEFAULT_PLACE, next()][next() as usize % 2],
            ]);
        }
    }
}

static COPY: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn run_copy(r: *mut Regs) {
    unsafe { crate::framework::call_game(r, COPY.load(Ordering::SeqCst)) };
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut tw = TestWorld::new();
    let w = tw.world();
    let copy = match unsafe {
        RedirectedCopy::new(game_addr(rva(0x40A9D1)), LEN, &ENTRY_BYTES, &[], &|_| None)
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    // Every reference of the copy to the game's world, to the test's (17 of them).
    let mut patched = 0;
    for (game, test) in w.pairs() {
        for offset in 0..LEN - 3 {
            let at = (copy.addr() + offset) as *const u32;
            if unsafe { at.read_unaligned() } == game {
                unsafe { copy.patch_u32(offset, test) };
                patched += 1;
            }
        }
    }
    if patched != 17 {
        t.check(false, || {
            format!("{patched} references to the world, not 17")
        });
        return t.finish();
    }
    COPY.store(copy.addr(), Ordering::SeqCst);
    let mut rng = 0x5EED_1234u32;
    let rust = rust_on(w);
    for case in 0..600 {
        let seed = rng;
        tw.fill(&mut rng);
        let start = tw.block.clone();
        let cmd_at = tw.at(T_COMMAND);
        let sound = unsafe { read(cmd_at + 4) } as u16 as i16 as i32 as u32;
        let before = Regs {
            eax: sound,
            esi: cmd_at,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { run_copy(&mut ro) };
        let theirs = tw.block.clone();
        tw.block.copy_from_slice(&start);
        let mine = rust(&before);
        let regs_ok = mine.as_ref().is_ok_and(|left| left.matches(&before, &ro));
        let differ: Vec<String> = (0..T_LEN)
            .filter(|&i| theirs[i] != tw.block[i])
            .take(12)
            .map(|i| format!("word {i:#x}: {:#x} / {:#x}", theirs[i], tw.block[i]))
            .collect();
        t.check(regs_ok && differ.is_empty(), || {
            format!(
                "case {case} (seed {seed:#x}), sound {sound:#x}: regs_ok={regs_ok} rust {mine:?}; \
                 original / rust: {}",
                differ.join(", ")
            )
        });
    }
    t.finish()
}
