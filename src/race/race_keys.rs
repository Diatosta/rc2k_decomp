//! `race::race_keys` (0x43A649): the race's view and sound keys, each frame.
//!
//! In:  nothing
//! Out: eax, esi (the view block), ebx (on the next-view key) as it leaves them; the rest
//!      kept (both callers go on with a `call` or a `mov`: no flags read)
//!
//! `drive::choose_shown_cars`; esi the first view's block (`off_63DBB0`), the second's
//! (`off_63DBB4`) with `byte_4F6B79` set. Then the first key down, in this order:
//! - `byte_4F6D17`: `drive::next_camera_view`, then `set_car_counter` with
//!   `dword_6138D4[view]` when the view (+0x688) is below 7;
//! - `byte_4F6D18` (not with one player): `dword_6238A4 ^= 3`, `set_both_counters` 0x127
//!   (now 1) or 0x128;
//! - `byte_4F6D19` / `byte_4F6D1A`: the volume down / up by 4 (`& !3`, held to 0 ..= 0x40):
//!   `g_masterVolume` then `sound::send_command_8` (counters 0x113 / 0x114); with
//!   `byte_4F6B79` set `dword_518100` then `sound::set_music_volume` (0x115 / 0x116; the
//!   "up" step starts from `dword_75568C`, a slip of the original's: `notes/bugs.md`);
//! - seven keys that flip a bit of the block's +0x220, each with `set_car_counter` (see
//!   `BIT_KEYS`);
//! - `byte_4F6D1B`: bit 0x40 of the first view's block, `set_both_counters` 0x125 / 0x126.
//!
//! Refuses (before anything) what `choose_shown_cars` refuses. Starts with a call:
//! private copy (every call relocated), manual patch. Shadow mode: the original runs (its
//! sounds sent), the memory put back, the Rust version runs without sending, the memory
//! and registers compared, the original's put back. Self-test: a copy whose two sound calls
//! go to stand-ins that note eax .. ecx, against the Rust version on the same, every key
//! with both views, players, volumes at the ends; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_write, hook_asm, rva};
use crate::race::frame::*;
use crate::race::level_init::{note, take_called};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, dispatch_replayed_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALLS: [usize; 21] = [
    0x0, 0x23, 0x5D, 0x82, 0xAB, 0xC3, 0xEC, 0x109, 0x132, 0x14A, 0x178, 0x1AA, 0x1DA, 0x20D,
    0x23D, 0x26D, 0x29D, 0x2CA, 0x2FD, 0x306, 0x31D,
];

pub static HOOK: RawHook = RawHook::new("race::race_keys", rva(0x43A649), thunk, &ORIGINAL)
    .private_copy(0x323)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn all(after: &Regs) -> Left {
    use crate::framework::{EAX, EBP, EBX, ECX, EDI, EDX, ESI};
    Left::nothing()
        .with(EAX, after.eax)
        .with(EBX, after.ebx)
        .with(ECX, after.ecx)
        .with(EDX, after.edx)
        .with(ESI, after.esi)
        .with(EDI, after.edi)
        .with(EBP, after.ebp)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let after = unsafe { race_keys(r, &mut key_sound_part)? };
    Ok(all(&after))
}

fn replay(r: &Regs) -> Result<Left, Refused> {
    let after = unsafe { race_keys(r, &mut |_, _| Ok(()))? };
    Ok(all(&after))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_replayed_state(&HOOK, r, call_original, &key_regions(), &rust, &replay);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

const PLAYERS: usize = rva(0x63DB94); // dword_63DB94
const ONE_CHOSEN: usize = rva(0x52A910); // dword_52A910
const OTHER_VOLUME: usize = rva(0x75568C); // dword_75568C

struct Case {
    key: Option<usize>,
    second: bool,
    players: u32,
    split: u32,
    volumes: [u32; 3],
    flags: u32,
    view: u32,
}

fn prepare(c: &Case) {
    unsafe {
        for k in KEY_BYTES {
            game_write::<u8>(rva(k), 0);
        }
        if let Some(k) = c.key {
            game_write::<u8>(rva(k), 1);
        }
        game_write::<u8>(rva(0x4F6B79), u8::from(c.second));
        game_write(PLAYERS, c.players);
        game_write(ONE_CHOSEN, 0u32);
        game_write(SPLIT_RVA, c.split);
        game_write(MASTER_VOLUME_RVA, c.volumes[0]);
        game_write(MUSIC_VOLUME_RVA, c.volumes[1]);
        game_write(OTHER_VOLUME, c.volumes[2]);
        let first = crate::framework::game_addr(rva(0x70F3EC)) as u32;
        for n in 0..8 {
            let b = first + n * 0xFE0;
            ((b + 0x220) as *mut u32).write_unaligned(c.flags);
            ((b + 0x688) as *mut u32).write_unaligned(c.view);
            ((b + 0x694) as *mut u32).write_unaligned(0);
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43A649,
            len: 0x323,
            entry: &[0xE8, 0x9C, 0xBA, 0x01, 0x00, 0x8B, 0x35, 0xB0],
            calls: &CALLS,
            jumps_out: &[],
        },
        &|c| {
            KEY_SOUND_CALLEES
                .iter()
                .find(|&&(_, k)| rva(k) == c)
                .map(|&(place, _)| crate::install::stand_ins::stand_in(place))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let regions = key_regions();
    let mut kept = regions.clone();
    for k in KEY_BYTES {
        kept.push((crate::framework::game_addr(rva(k)) as u32, 1));
    }
    kept.extend([at(PLAYERS), at(ONE_CHOSEN), at(OTHER_VOLUME)]);
    let saved = capture(&kept);

    let mut keys: Vec<Option<usize>> = vec![None, Some(0x4F6D17), Some(0x4F6D18)];
    keys.extend([0x4F6D19, 0x4F6D1A, 0x4F6D1B].map(Some));
    keys.extend(BIT_KEYS.map(Some));
    let mut cases = Vec::new();
    for second in [false, true] {
        for &key in &keys {
            for (players, split, volumes, flags, view) in [
                (2, 1, [0x20, 0x22, 0x31], 0, 0),
                (1, 2, [0, 0x40, 0x3F], 0xFF, 6),
                (2, 2, [0x8000_0003, 3, 0xFFFF_FFFE], 0x55, 7),
            ] {
                cases.push(Case {
                    key,
                    second,
                    players,
                    split,
                    volumes,
                    flags,
                    view,
                });
            }
        }
    }
    for c in &cases {
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        restore(&kept, &saved);
        prepare(c);
        take_called();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        restore(&kept, &saved);
        prepare(c);
        let mine = unsafe {
            race_keys(&Regs::SENTINEL, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            take_called(),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "key {:x?} second {} players {} split {} volumes {:x?} flags {:#x} view {}: \
                 original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
                c.key,
                c.second,
                c.players,
                c.split,
                c.volumes,
                c.flags,
                c.view,
                theirs.0,
                theirs.1,
                mine.0,
                mine.1
            )
        });
    }
    restore(&kept, &saved);
    t.finish()
}
