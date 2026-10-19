//! `physics::queue_pace_note` (0x4418B3): the pace notes group called at checkpoint edx
//! (see `race/pace_notes.rs`): with groups (`dword_615E14` not 0), the first group whose
//! checkpoint (`dword_619E18`) is edx; its text (`byte_61BE18` from `dword_61AE18[group]`)
//! code by code up to a NUL: each below 0x54 cued (`physics::cue_listed_car_sound_last`,
//! inside `pusha` / `popa`) and queued as a car event (`physics::queue_listed_event`,
//! ebx the code); one of 0x54 or more ends it with "!!! invalid pace data", "data", the
//! code, "ecx", the group, "pointer", its place logged (level 1, each in `pusha` /
//! `popa`).
//!
//! In:  edx the checkpoint
//! Out: no groups: every register kept; none at edx: ecx the groups' count; else ecx the
//!      group, esi the text's place where it stopped, ebx that code (0 or 0x54 ..); the
//!      rest kept; no flags read (the caller's `pusha` / `popa`)
//!
//! Caller: `physics::pace_notes_step` (inside `pusha` / `popa`). Its calls by place (this
//! file's `pace_part`): 0 `logfile::log_write_l1`, 1 `logfile::log_int_l1`, 2
//! `logfile::log_int_l1_nl`, 3 `physics::cue_listed_car_sound_last`, 4
//! `physics::queue_listed_event`, 5 this function, 6 `physics::expire_car_events`.
//! Refuses (before anything) more than 0x400 groups (the scan reads past `dword_619E18`)
//! and a text place past the 0x4000 bytes (read past the text; checked at each code).
//! Shadow mode runs the original alone (sound cues and car events must not be queued
//! twice). Self-test: a copy whose calls go to stand-ins that note them, against the Rust
//! version with the same, on groups of the test's in the game's tables (put back).

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::queue_pace_note", rva(0x4418B3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

pub const LOG_WRITE: u32 = 0;
pub const LOG_INT: u32 = 1;
pub const LOG_INT_NL: u32 = 2;
pub const CUE: u32 = 3;
pub const QUEUE: u32 = 4;
pub const PACE: u32 = 5;
pub const EXPIRE: u32 = 6;
/// The places' functions, as IDA shows them.
pub const PLACES: [usize; 7] = [
    0x401427, 0x401472, 0x401476, 0x45D1D3, 0x44194D, 0x4418B3, 0x44196B,
];

const GROUPS: usize = 0x615E14; // dword_615E14
const GROUP_CHECKPOINTS: usize = 0x619E18; // dword_619E18
const GROUP_TEXTS: usize = 0x61AE18; // dword_61AE18
const TEXT: usize = 0x61BE18; // byte_61BE18
const TEXT_LEN: u32 = 0x4000;
const MOST_GROUPS: u32 = 0x400;
/// "!!! invalid pace data", "data", "ecx", "pointer".
const WORDS: [usize; 4] = [0x4DBB70, 0x4DBB86, 0x4DBB8B, 0x4DBB8F];

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

/// The steps on `r`.
pub fn queue_pace_note_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let groups = g(GROUPS);
    if groups == 0 {
        return Ok(());
    }
    if groups > MOST_GROUPS {
        return Err(BadInput("more than 0x400 pace note groups").into());
    }
    r.ecx = 0;
    while r.edx != g(GROUP_CHECKPOINTS + 4 * r.ecx as usize) {
        r.ecx += 1;
        if r.ecx == groups {
            return Ok(());
        }
    }
    r.esi = g(GROUP_TEXTS + 4 * r.ecx as usize);
    loop {
        if r.esi >= TEXT_LEN {
            return Err(BadInput("a pace note text place past byte_61BE18's 0x4000").into());
        }
        r.ebx = unsafe { game_read::<u8>(rva(TEXT) + r.esi as usize) } as u32;
        if r.ebx == 0 {
            return Ok(());
        }
        if r.ebx >= 0x54 {
            let word = |n: usize| game_addr(rva(WORDS[n])) as u32;
            for (place, eax) in [
                (LOG_WRITE, word(0)),
                (LOG_WRITE, word(1)),
                (LOG_INT, r.ebx),
                (LOG_WRITE, word(2)),
                (LOG_INT_NL, r.ecx),
                (LOG_WRITE, word(3)),
                (LOG_INT_NL, r.esi),
            ] {
                let mut c = Regs { eax, ..*r };
                part(place, &mut c)?;
            }
            return Ok(());
        }
        let mut c = *r;
        part(CUE, &mut c)?;
        part(QUEUE, r)?;
        r.esi = r.esi.wrapping_add(1);
    }
}

/// The calls in the game: the Rust versions.
pub fn pace_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    unsafe {
        match place {
            LOG_WRITE => log_write(r.eax as *const u8, false, 1),
            LOG_INT | LOG_INT_NL => log_int(r.eax as i32, place == LOG_INT_NL, 1),
            CUE => crate::physics::cue_listed_car_sound_last::cue_listed_car_sound_last_in(
                r,
                &mut crate::physics::sound_cues::game_part,
            )?,
            QUEUE => crate::physics::queue_listed_event::queue_listed_event(r.ebx)?.apply(r),
            PACE => queue_pace_note_in(r, &mut pace_part)?,
            _ => crate::physics::expire_car_events::expire_car_events_at(
                &crate::physics::queue_car_event::QueuePlaces::game(),
            )?
            .apply(r),
        }
    }
    Ok(())
}

/// The pace notes' hooks' dispatch: the original alone in shadow mode, else `rust` with
/// the game's calls.
pub fn dispatch_pace(
    hook: &RawHook,
    r: &mut Regs,
    call_original: unsafe extern "C" fn(*mut Regs),
    rust: fn(&mut Regs, Part) -> Result<(), Refused>,
) {
    hook.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let mut g = *r;
    match rust(&mut g, &mut pace_part) {
        Ok(()) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_pace(&HOOK, r, call_original, queue_pace_note_in);
}

// ---- for the self-tests ----

static CALLED: Mutex<Vec<(u32, [u32; 7])>> = Mutex::new(Vec::new());

/// The stand-ins: each call noted with its registers, which it leaves as they came.
pub fn handler(place: u32, r: &mut Regs) {
    CALLED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]));
}

pub fn take_called() -> Vec<(u32, [u32; 7])> {
    std::mem::take(&mut *CALLED.lock().unwrap())
}

/// A copy of one of the pace notes' functions whose calls (at `calls`) go to the
/// stand-ins by `PLACES`.
pub fn copy_of(
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
) -> Result<RedirectedCopy, &'static str> {
    set_handler(handler);
    unsafe {
        RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|c| {
            PLACES
                .iter()
                .position(|&k| rva(k) == c)
                .map(|n| stand_in(n as u32))
        })
    }
}

/// One case: `fill` then the copy, `fill` then `rust` with the stand-ins; the calls, the
/// registers and the memory compared.
pub fn both(
    t: &mut SelfTest,
    what: &str,
    copy: &RedirectedCopy,
    entry: Regs,
    fill: &dyn Fn(),
    all: &[Region],
    rust: fn(&mut Regs, Part) -> Result<(), Refused>,
) {
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    fill();
    take_called();
    let mut ro = entry;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = (Some(regs(&ro)), take_called(), capture(all));
    fill();
    let mut g = entry;
    let ok = rust(&mut g, &mut |p, r| {
        handler(p, r);
        Ok(())
    })
    .is_ok();
    let mine = (ok.then(|| regs(&g)), take_called(), capture(all));
    t.check(theirs == mine, || {
        let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
        format!(
            "{what}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
            theirs.0, theirs.1, mine.0, mine.1
        )
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x4418B3,
        0x9A,
        &[0x83, 0x3D, 0x14, 0x5E, 0x61, 0x00, 0x00],
        &[0x40, 0x4D, 0x56, 0x63, 0x6C, 0x79, 0x82, 0x8B, 0x91],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let a = |x: usize| game_addr(rva(x)) as u32;
    let all = [(a(0x615E10), 0xA008)];
    let saved = capture(&all);
    // Five groups at checkpoints 3, 7, 7, 12, 20; texts with codes of each kind.
    let texts: [&[u8]; 5] = [
        &[0x05, 0x1F, 0x20, 0x29, 0x00],
        &[0x53, 0x01, 0x00],
        &[0x02, 0x00],
        &[0x10, 0x54, 0x07, 0x00],
        &[0x00],
    ];
    for groups in [0u32, 1, 3, 5] {
        for edx in [3u32, 7, 12, 20, 21] {
            let fill = || unsafe {
                use crate::framework::game_write;
                restore(&all, &saved);
                game_write(rva(GROUPS), groups);
                let mut at = 0x100u32;
                for (n, (cp, text)) in [3u32, 7, 7, 12, 20].iter().zip(texts).enumerate() {
                    game_write(rva(GROUP_CHECKPOINTS) + 4 * n, *cp);
                    game_write(rva(GROUP_TEXTS) + 4 * n, at);
                    for (k, &b) in text.iter().enumerate() {
                        game_write::<u8>(rva(TEXT) + (at as usize) + k, b);
                    }
                    at += 0x20;
                }
            };
            both(
                &mut t,
                &format!("groups {groups}, edx {edx}"),
                &copy,
                Regs {
                    edx,
                    ..Regs::SENTINEL
                },
                &fill,
                &all,
                queue_pace_note_in,
            );
        }
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
