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
//! One caller, `sound::set_up_race_sounds`. Refuses a car number of 8 or more, a missing
//! object, a kind of 0x20 or more and a list of names that does not hold 95 of at most 0x100
//! bytes.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{Regs, game_addr, rva};
use crate::sound::queue::{Queue, START_3, send_command, sent};

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
        // Converted: `rc2k_core::sound::race::voices` and `sets`.
        use rc2k_core::sound::race::{Set, Voice, sets, voices};
        let list = |v: Voice| match v {
            Voice::Male => game_addr(MALE_RVA),
            Voice::Female => game_addr(FEMALE_RVA),
        };
        let (a, b) = voices(dword(first + 0x298), dword(second + 0x298));
        let (mut ebx, mut edx) = (list(a), list(b));
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
        let (first_kind, second_kind) = kinds.unwrap_or((0, None));
        for set in sets(
            kinds.is_none(),
            first_kind as u32,
            second_kind.unwrap_or(0) as u32,
            count,
            other_count,
        ) {
            send(match set {
                Set::Replay => game_addr(REPLAY_SET_RVA) as u32,
                Set::FirstKind(k) => dword(game_addr(KINDS_RVA) + k as usize * 8),
                Set::First(n) => game_addr(rva(FIRST_SETS[n])) as u32,
                Set::SecondKind(k) => dword(game_addr(KINDS_RVA) + 4 + k as usize * 8),
                Set::Second(n) => game_addr(rva(SECOND_SETS[n])) as u32,
                Set::Last => game_addr(LAST_SET_RVA) as u32,
            });
        }
        Ok(())
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    let done = unsafe {
        load_race_sounds(&entry, &Places::game(), &mut |with| {
            send_command(&Queue::game(), START_3, sent(with))
        })
    };
    if let Err(e) = done {
        return Err(e.into());
    }
    Ok(())
}
