//! `sound::load_race_sounds` (0x40EED8): names the co-driver's speech files and tells the
//! audio thread which sets of sounds a race needs.
//!
//! In:  ebp goes into the commands as it is
//! Out: every register preserved
//! The two cars' sound objects are `off_512A10[dword_514F70]` and
//! `off_512A10[dword_514F74]`. The first's dword +0x298 says whose voice it has: 0 the
//! list of 95 file names at 0x507C1A (`files\%speech\m\..`), else the list at 0x508914
//! (`..\f\..`); the second car has the other list, or the same one if its +0x298 is the
//! first's. The sample records 0xED .. 0x1AA of `unk_50A7A4` (0x40 bytes each) get the
//! names at their +0xC: the odd ones the first car's, the even ones the second's.
//! Then the sets, each a command 3 with the set's address in eax: in a replay
//! (`dword_75B108` not 0) only `unk_512340`; else the first car's kind
//! (`off_512360[its +4 * 8]`), `unk_511CA8`, `unk_512268`, `unk_511D04`; unless
//! `dword_514F78` is 1 the second car's kind (`off_512364[its +4 * 8]`), `unk_512014`,
//! `unk_512070`; and `unk_5121EC` unless `dword_514F78` is `dword_514F7C`.
//! The commands carry the loop's leftovers in the other registers, reproduced.
//! What the two counts and "a replay" are is a reading.
//! One caller, `sound::set_up_race_sounds`. Shadow mode runs the original alone.
//! Self-test: a redirected copy whose variables, objects and sample records are the
//! test's and whose commands note their registers.
//! Refuses a car number of 8 or more, a missing object, a kind of 0x20 or more and a
//! list of names that does not hold 95 of at most 0x100 bytes.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::sound::queue::{Queue, START_3, send_command, sent};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::load_race_sounds", rva(0x40EED8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const MALE_RVA: usize = rva(0x507C1A); // aFilesSpeechMCo
const FEMALE_RVA: usize = rva(0x508914); // aFilesSpeechFCo
const KINDS_RVA: usize = rva(0x512360); // off_512360: two sets a kind of car
const REPLAY_SET_RVA: usize = rva(0x512340); // unk_512340
const FIRST_SETS: [usize; 3] = [0x511CA8, 0x512268, 0x511D04];
const SECOND_SETS: [usize; 2] = [0x512014, 0x512070];
const LAST_SET_RVA: usize = rva(0x5121EC); // unk_5121EC

const FIRST_SAMPLE: usize = 0xED;
const NAMES: usize = 95;
const SAMPLE_LEN: usize = 0x40;
const OBJECTS: u32 = 8;
const KINDS: u32 = 0x20;

/// The variables, as addresses: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct Places {
    /// dword_514F70, dword_514F74: the two cars' numbers.
    pub first: usize,
    pub second: usize,
    /// dword_514F78, dword_514F7C.
    pub count: usize,
    pub other_count: usize,
    /// dword_75B108.
    pub replay: usize,
    /// off_512A10: eight sound objects.
    pub objects: usize,
    /// unk_50A7A4: the sample records.
    pub samples: usize,
}

const PLACES: [usize; 7] = [
    0x514F70, 0x514F74, 0x514F78, 0x514F7C, 0x75B108, 0x512A10, 0x50A7A4,
];

impl Places {
    fn from(addresses: [usize; 7]) -> Self {
        let [first, second, count, other_count, replay, objects, samples] = addresses;
        Self {
            first,
            second,
            count,
            other_count,
            replay,
            objects,
            samples,
        }
    }

    pub fn game() -> Self {
        Self::from(PLACES.map(|at| game_addr(rva(at))))
    }
}

unsafe fn dword(at: usize) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The lengths of a list's 95 names.
unsafe fn lengths(list: usize) -> Result<Vec<u32>, BadInput> {
    let mut at = list;
    let mut lengths = Vec::with_capacity(NAMES);
    for _ in 0..NAMES {
        let length = (0..0x100usize)
            .find(|&n| unsafe { ((at + n) as *const u8).read() } == 0)
            .ok_or(BadInput("a speech file's name without its end"))?;
        lengths.push(length as u32);
        at += length + 1;
    }
    Ok(lengths)
}

// ---- Rust version ----

/// `send` gets the registers each command 3 is sent with.
pub unsafe fn load_race_sounds(
    entry: &Regs,
    places: &Places,
    send: &mut dyn FnMut(&Regs),
) -> Result<(), BadInput> {
    unsafe {
        let object = |number: usize| -> Result<usize, BadInput> {
            let number = dword(number);
            if number >= OBJECTS {
                return Err(BadInput("a car's number past the sound objects"));
            }
            match dword(places.objects + number as usize * 4) {
                0 => Err(BadInput("no sound object for a car")),
                object => Ok(object as usize),
            }
        };
        let (first, second) = (object(places.first)?, object(places.second)?);
        let kind = |object: usize| match dword(object + 4) {
            kind if kind < KINDS => Ok(kind as usize),
            _ => Err(BadInput("a car's kind past the table of sound sets")),
        };
        let replay = dword(places.replay) != 0;
        let (count, other_count) = (dword(places.count), dword(places.other_count));
        let kinds = if replay {
            None
        } else {
            let second = if count != 1 {
                Some(kind(second)?)
            } else {
                None
            };
            Some((kind(first)?, second))
        };
        let voice = dword(first + 0x298);
        let (male, female) = (game_addr(MALE_RVA), game_addr(FEMALE_RVA));
        let (mut ebx, mut edx) = if voice == 0 {
            (male, female)
        } else {
            (female, male)
        };
        if voice == dword(second + 0x298) {
            edx = ebx;
        }
        let (of_first, of_second) = (lengths(ebx)?, lengths(edx)?);
        // Nothing has been written or sent up to here.
        let (mut esi, mut ecx) = (0, 0);
        for n in 0..NAMES {
            esi = places.samples + (FIRST_SAMPLE + n * 2) * SAMPLE_LEN;
            ((esi + 0x0C) as *mut u32).write_unaligned(ebx as u32);
            ((esi + 0x4C) as *mut u32).write_unaligned(edx as u32);
            ebx += of_first[n] as usize + 1;
            ecx = of_second[n];
            edx += ecx as usize + 1;
        }
        let mut send = |set: u32| {
            send(&Regs {
                eax: set,
                ebx: ebx as u32,
                ecx,
                edx: edx as u32,
                esi: esi as u32,
                edi: second as u32,
                ..*entry
            })
        };
        let set = |at: usize| game_addr(rva(at)) as u32;
        let Some((first_kind, second_kind)) = kinds else {
            send(game_addr(REPLAY_SET_RVA) as u32);
            return Ok(());
        };
        send(dword(game_addr(KINDS_RVA) + first_kind * 8));
        for at in FIRST_SETS {
            send(set(at));
        }
        if let Some(second_kind) = second_kind {
            send(dword(game_addr(KINDS_RVA) + 4 + second_kind * 8));
            for at in SECOND_SETS {
                send(set(at));
            }
        }
        if count != other_count {
            send(game_addr(LAST_SET_RVA) as u32);
        }
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    let done = unsafe {
        load_race_sounds(&entry, &Places::game(), &mut |with| {
            send_command(&Queue::game(), START_3, sent(with))
        })
    };
    if let Err(e) = done {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

const LEN: usize = 0x11B;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xA1, 0x70, 0x4F, 0x51, 0x00, 0x8B, 0x34];
/// `strings::text_length` twice, then the nine senders of command 3.
const CALLS: [usize; 11] = [
    0x51, 0x5C, 0x94, 0x9F, 0xAA, 0xB5, 0xD9, 0xE4, 0xEF, 0x107, 0x114,
];
const SEND_3_RVA: usize = rva(0x409BBF);
/// How often the code names each of `PLACES`.
const NAMED: [usize; 7] = [2, 2, 2, 1, 1, 4, 1];
/// A test's sound object: its kind at +4 and its voice at +0x298.
const OBJECT_LEN: usize = 0x2A0;

static SENT: Mutex<Vec<[u32; 7]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    SENT.lock().unwrap().push(sent(r));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x40EED8)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == SEND_3_RVA).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    // The test's memory: five variables, a table of eight objects, the objects, and the
    // sample records up to the last one written.
    let mut vars = [0u32; 5];
    let mut table = vec![0u32; OBJECTS as usize];
    let mut objects = vec![0u8; OBJECT_LEN * 4];
    let samples_len = (FIRST_SAMPLE + NAMES * 2 + 1) * SAMPLE_LEN;
    let mut samples = vec![0u8; samples_len];
    for (n, entry) in table.iter_mut().take(4).enumerate() {
        *entry = objects.as_mut_ptr() as u32 + (n * OBJECT_LEN) as u32;
    }
    let mut addresses = [0usize; 7];
    for (n, var) in vars.iter_mut().enumerate() {
        addresses[n] = var as *mut u32 as usize;
    }
    addresses[5] = table.as_ptr() as usize;
    addresses[6] = samples.as_mut_ptr() as usize;
    let places = Places::from(addresses);
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let real = PLACES.map(|at| game_addr(rva(at)) as u32);
        let mut patches = Vec::new();
        let mut seen = [0usize; 7];
        let mut at = 0;
        while at + 4 <= LEN {
            let named = u32::from_le_bytes(code[at..at + 4].try_into().unwrap());
            match real.iter().position(|&address| address == named) {
                Some(n) => {
                    seen[n] += 1;
                    patches.push((at, addresses[n] as u32));
                    at += 4;
                }
                None => at += 1,
            }
        }
        if seen != NAMED {
            return t.skip("the function does not name its variables as the test expects");
        }
        for (at, value) in patches {
            copy.patch_u32(at, value);
        }
    }
    // (first, second, count, other count, replay), then each object's (kind, voice).
    type Case = ([u32; 5], [(u32, u32); 4]);
    let plain = [(0, 0), (1, 1), (0x1F, 0), (7, 5)];
    let cases: [Case; 12] = [
        ([0, 1, 2, 2, 0], plain),
        ([0, 1, 1, 1, 0], plain),
        ([0, 1, 1, 2, 0], plain),
        ([1, 0, 2, 1, 0], plain),
        ([0, 2, 2, 2, 0], plain),
        ([1, 3, 2, 0, 0], plain),
        ([3, 1, 4, 4, 0], plain),
        ([2, 2, 2, 2, 0], plain),
        ([0, 1, 2, 2, 1], plain),
        ([1, 0, 1, 2, 0xFFFF_FFFF], plain),
        ([0, 1, 2, 2, 0], [(3, 2), (4, 2), (0, 0), (0, 0)]),
        ([0, 1, 0, 1, 0], [(3, 0x8000_0000), (4, 0), (0, 0), (0, 0)]),
    ];
    for (values, kinds) in cases {
        let before = Regs::SENTINEL;
        let mut run = |rust: bool| unsafe {
            vars.copy_from_slice(&values);
            for (n, (kind, voice)) in kinds.into_iter().enumerate() {
                let object = objects.as_mut_ptr().add(n * OBJECT_LEN);
                (object.add(4) as *mut u32).write_unaligned(kind);
                (object.add(0x298) as *mut u32).write_unaligned(voice);
            }
            samples.fill(0xAA);
            SENT.lock().unwrap().clear();
            set_handler(handler);
            let ended = if rust {
                match load_race_sounds(&before, &places, &mut |with| {
                    SENT.lock().unwrap().push(sent(with))
                }) {
                    Ok(()) => 0,
                    Err(_) => 2,
                }
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                !ro.preserved_except(&before, 0) as u32
            };
            (ended, samples.clone(), SENT.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 == 0, || {
            format!(
                "variables {values:x?}, objects {kinds:x?}: original ended {} sent {:x?}; rust ended {} sent {:x?}; first different sample byte {:?}",
                theirs.0,
                theirs.2,
                mine.0,
                mine.2,
                theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b),
            )
        });
    }
    t.finish()
}
