//! `audio::update_mixer_voices` (0x40BC05, a chunk of 0x40AC77 entered by `jnz`): the
//! turn's update of the voices on the game's own mixer.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The playing list: a voice with dirty bits gets, on its channel, as marked: the step
//! (frequency `>>` the halvings, over `g_mixRate`, as 32.32) for 4; the left and right
//! volumes for 1 or 2 (volume x factor, times 64 - pan and 64 for a pan of 0 and up, 64 and
//! 64 + pan below, each `>> 6`; with mode bit 0x2000000 and a negative pan sign the right
//! negated); for a restart (0x20) or a voice not yet sounding, the channel's position and
//! fraction 0 and its format, data, length (the voice's frames) and kind, the voice's
//! state 2; dirty bits 0. A voice that does not loop whose channel is no longer used (the
//! mixer ended it) has its channel freed and 0, goes to the stopping list's front, and
//! `g_channelsUsed` is one less. The stopping list: each voice's dirty bits + 0xF, state
//! 0, channel freed and 0, and the list emptied. `audio::mix`. The playing list again for
//! the voices the mix ended.
//! Refuses, before anything: lists without their end; a playing voice without a channel; no
//! mixing rate with a frequency to set (the original divides by 0); and a voice whose
//! volume or pan changes while its sound has ended: the original's ebx (the list's link)
//! then holds the pan, and it writes the link through it (section 16).
//! Shadow mode runs the original alone. Self-test: a redirected copy of the chunk whose
//! variables are the test's and whose mix notes itself, 200 states by lot.

use crate::audio::mixer::MIX_RATE_RVA;
use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use rc2k_core::audio::voices::Lists;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "audio::update_mixer_voices",
    rva(0x40BC05),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

#[derive(Clone, Copy)]
pub struct Vars {
    pub rate: u32,
    pub mode: u32,
    pub playing: u32,
    pub stopping: u32,
    pub used: u32,
}

impl Vars {
    pub fn game() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            rate: a(MIX_RATE_RVA),
            mode: a(SOUND_MODE_RVA),
            playing: a(PLAYING_RVA),
            stopping: a(STOPPING_RVA),
            used: a(CHANNELS_USED_RVA),
        }
    }
}

unsafe fn lists(v: &Vars) -> Lists {
    unsafe {
        Lists {
            playing: read(v.playing),
            stopping: read(v.stopping),
            channels_used: read(v.used),
            channel_count: 0,
        }
    }
}

unsafe fn store(v: &Vars, l: &Lists) {
    unsafe {
        write(v.playing, l.playing);
        write(v.stopping, l.stopping);
        write(v.used, l.channels_used);
    }
}

/// `mix`: `audio::mix`.
pub unsafe fn update_mixer_voices(
    v: &Vars,
    mix: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::mixer_voices`; the lists stored for the mix, which
    // reads them, and read again after it.
    use rc2k_core::audio::mixer_voices::{after_mix, before_mix};
    unsafe {
        let rate = read(v.rate);
        let mode = read(v.mode);
        let mut pool = GamePool::new(&World::game());
        let mut l = lists(v);
        before_mix(&mut pool, &mut l, rate, mode)?;
        store(v, &l);
        mix()?;
        let mut l = lists(v);
        after_mix(&mut pool, &mut l, rate, mode)?;
        store(v, &l);
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let mut mix = || -> Result<(), Refused> {
        let vars = crate::audio::mix::Vars::game();
        unsafe { crate::audio::mix::mix(&vars, &mut crate::audio::mix::game_part) }?;
        Ok(())
    };
    if let Err(e) = unsafe { update_mixer_voices(&Vars::game(), &mut mix) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

static MIXED: Mutex<u32> = Mutex::new(0);
/// Which channels the test's mix ends, by bit.
static ENDS: Mutex<(u32, u32)> = Mutex::new((0, 0));

fn handler(_: u32, _: &mut Regs) {
    *MIXED.lock().unwrap() += 1;
    let (base, ends) = *ENDS.lock().unwrap();
    for n in 0..10u32 {
        if ends & (1 << n) != 0 {
            unsafe { write(base + n * std::mem::size_of::<MixerChannel>() as u32, 0) };
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    const N: usize = 10;
    let mut voices = vec![Voice::default(); N];
    // The channels are only reached through the voices' pointers: one of the test's each,
    // their `used` dword first.
    let mut channels = vec![MixerChannel::default(); N];
    let mut vars = Box::new([0u32; 5]);
    let at = vars.as_mut_ptr() as u32;
    let v = Vars {
        rate: at,
        mode: at + 4,
        playing: at + 8,
        stopping: at + 12,
        used: at + 16,
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40BC05,
            len: 0x1C0,
            entry: &[0x60, 0x8D, 0x1D, 0x40, 0x83, 0x4F, 0x00],
            calls: &[0x6C],
            jumps_out: &[],
        },
        &|callee| (callee == rva(0x40BDC5)).then(|| stand_in(0)),
        &[
            (0x5068D4, v.rate),
            (0x4F82BC, v.mode),
            (0x4F8340, v.playing),
            (0x4F8344, v.stopping),
            (0x506350, v.used),
        ],
        12,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let vbase = voices.as_mut_ptr() as u32;
    let cbase = channels.as_mut_ptr() as u32;
    let csize = std::mem::size_of::<MixerChannel>() as u32;
    let vat = |n: usize| vbase + (n * std::mem::size_of::<Voice>()) as u32;
    let mut rng = 0xA11C_E001u32;
    let mut case = 0;
    while case < 200 {
        let seed = rng;
        let mut next = || {
            rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            rng >> 8
        };
        let setup: Vec<(u32, Voice, u32)> = (0..N)
            .map(|n| {
                let list = next() % 3;
                let x = Voice {
                    format: [0x110u32, 0x221, 0x122, 0x210][next() as usize % 4],
                    data: 0x1000 * n as u32,
                    frames: next() % 0x1000,
                    kind: [0, 1, 3][next() as usize % 3],
                    state: next() % 3,
                    dirty: next() % 0x40,
                    volume: next() % 0x41,
                    volume_factor: next() % 0x41,
                    pan: (next() % 0x81) as i32 - 0x40,
                    pan_sign: if next() % 2 == 0 { 0 } else { -1 },
                    frequency: 100 + next() % 50_000,
                    ..Voice::default()
                };
                (list, x, [1u32, 1, 0][next() as usize % 3])
            })
            .collect();
        let mode = [0x4_1220u32, 0x204_1220][next() as usize % 2];
        let ends = next() & 0x3FF;
        // Only states the Rust version takes (the original crashes on the others).
        if setup
            .iter()
            .any(|(l, x, used)| *l == 0 && x.dirty & 3 != 0 && x.kind != 1 && *used != 1)
        {
            continue;
        }
        case += 1;
        let prepare =
            |voices: &mut Vec<Voice>, channels: &mut Vec<MixerChannel>, vars: &mut [u32; 5]| {
                let (mut playing, mut stopping) = (Vec::new(), Vec::new());
                for (n, (l, x, used)) in setup.iter().enumerate() {
                    voices[n] = *x;
                    channels[n] = MixerChannel {
                        used: *used,
                        ..MixerChannel::default()
                    };
                    voices[n].channel = if *l == 0 || n % 3 == 0 {
                        cbase + n as u32 * csize
                    } else {
                        0
                    };
                    match l {
                        0 => playing.push(n),
                        1 => stopping.push(n),
                        _ => {}
                    }
                }
                for l in [&playing, &stopping] {
                    for (i, &n) in l.iter().enumerate() {
                        voices[n].next = l.get(i + 1).map_or(0, |&m| vat(m));
                    }
                }
                *vars = [
                    [22050u32, 44100][case % 2],
                    mode,
                    playing.first().map_or(0, |&n| vat(n)),
                    stopping.first().map_or(0, |&n| vat(n)),
                    playing.len() as u32,
                ];
                *MIXED.lock().unwrap() = 0;
                // The test's mix ends the channels whose bit is set.
                *ENDS.lock().unwrap() = (cbase, ends);
            };
        let run = |rust: bool,
                   voices: &mut Vec<Voice>,
                   channels: &mut Vec<MixerChannel>,
                   vars: &mut [u32; 5]| {
            prepare(voices, channels, vars);
            let mut ro = Regs::SENTINEL;
            let done = if rust {
                unsafe {
                    update_mixer_voices(&v, &mut || {
                        handler(0, &mut Regs::default());
                        Ok(())
                    })
                }
                .map(|_| true)
            } else {
                unsafe { call_game(&mut ro, copy.addr()) };
                Ok(ro.preserved_except(&Regs::SENTINEL, 0))
            };
            (
                done,
                voices.clone(),
                channels.clone(),
                // Volatile: the code under test wrote it behind the reference.
                unsafe { std::ptr::from_mut(vars).read_volatile() },
                *MIXED.lock().unwrap(),
            )
        };
        let theirs = run(false, &mut voices, &mut channels, &mut vars);
        let mine = run(true, &mut voices, &mut channels, &mut vars);
        let ok = matches!(theirs.0, Ok(true))
            && mine.0.is_ok()
            && theirs.1 == mine.1
            && theirs.2 == mine.2
            && theirs.3 == mine.3
            && theirs.4 == mine.4;
        t.check(ok, || {
            format!(
                "case {case} (seed {seed:#x}), mode {mode:#x}: rust {:?}; original {:x?}; rust {:x?}",
                mine.0, theirs.3, mine.3
            )
        });
    }
    t.finish()
}
