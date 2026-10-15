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
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
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

const MODE_SURROUND: u32 = 0x200_0000;
const NEXT: u32 = std::mem::offset_of!(Voice, next) as u32;

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

unsafe fn list(head: u32) -> Result<Vec<u32>, Refused> {
    let mut out = Vec::new();
    let mut at = unsafe { read(head) };
    while at != 0 {
        out.push(at);
        if out.len() > VOICES as usize {
            return Err(BadInput("a list of voices without its end").into());
        }
        at = unsafe { voice(at).next };
    }
    Ok(out)
}

unsafe fn apply(at: u32, rate: u32, mode: u32) {
    unsafe {
        let x = voice(at);
        let c = channel(x.channel);
        if x.dirty & DIRTY_FREQUENCY != 0 {
            let f = x.frequency >> (x.format & 0xF);
            c.step = f / rate;
            c.step_fraction = ((((f % rate) as u64) << 32) / rate as u64) as u32;
        }
        if x.dirty & (DIRTY_VOLUME | DIRTY_PAN) != 0 {
            let volume = (x.volume as i32).wrapping_mul(x.volume_factor as i32);
            let (l, r) = if x.pan >= 0 {
                (0x40i32.wrapping_sub(x.pan), 0x40)
            } else {
                (0x40, 0x40i32.wrapping_add(x.pan))
            };
            c.left = l.wrapping_mul(volume) >> 6;
            let mut right = r.wrapping_mul(volume) >> 6;
            if mode & MODE_SURROUND != 0 && x.pan_sign < 0 {
                right = right.wrapping_neg();
            }
            c.right = right;
        }
        if x.dirty & DIRTY_RESTART != 0 || x.state != 2 {
            c.position = 0;
            c.position_fraction = 0;
            c.format = x.format;
            c.data = x.data;
            c.length = x.frames;
            c.kind = x.kind;
            x.state = 2;
        }
        x.dirty = 0;
    }
}

/// The playing list's voices that ended (not looping, channel no longer used) to the
/// stopping list; `with_apply`: the dirty bits carried out first.
unsafe fn playing_pass(v: &Vars, rate: u32, mode: u32, with_apply: bool) {
    unsafe {
        let mut link = v.playing;
        loop {
            let at = read(link);
            if at == 0 {
                return;
            }
            if with_apply && voice(at).dirty != 0 {
                apply(at, rate, mode);
            }
            let x = voice(at);
            if x.kind != 1 && channel(x.channel).used != 1 {
                write(x.channel, 0);
                x.channel = 0;
                write(link, x.next);
                x.next = read(v.stopping);
                write(v.stopping, at);
                write(v.used, read(v.used).wrapping_sub(1));
                continue;
            }
            link = at + NEXT;
        }
    }
}

/// `mix`: `audio::mix`.
pub unsafe fn update_mixer_voices(
    v: &Vars,
    mix: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let rate = read(v.rate);
        let mode = read(v.mode);
        for at in list(v.playing)? {
            let x = voice(at);
            if x.channel == 0 {
                return Err(BadInput("a playing voice without its channel").into());
            }
            if x.dirty & DIRTY_FREQUENCY != 0 && rate == 0 {
                return Err(BadInput("no mixing rate (the original divides by 0)").into());
            }
            if x.dirty & (DIRTY_VOLUME | DIRTY_PAN) != 0
                && x.kind != 1
                && channel(x.channel).used != 1
            {
                return Err(BadInput(
                    "a changed volume or pan on a voice that ended (the original writes through the pan)",
                )
                .into());
            }
        }
        list(v.stopping)?;
        playing_pass(v, rate, mode, true);
        let mut link = v.stopping;
        loop {
            let at = read(link);
            if at == 0 {
                break;
            }
            let x = voice(at);
            x.dirty |= 0xF;
            x.state = 0;
            if x.channel != 0 {
                write(x.channel, 0);
                x.channel = 0;
            }
            write(link, 0);
            link = at + NEXT;
        }
        mix()?;
        list(v.playing)?;
        playing_pass(v, rate, mode, false);
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
