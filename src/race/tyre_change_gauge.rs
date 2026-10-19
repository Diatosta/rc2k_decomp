//! `race::tyre_change_gauge` (0x41C1D6): the roadside tyre change's gauge, timed by a
//! button (a reading: `input_device_read`'s ebx low byte, the device's +0x4E8). The state
//! is the device's (`g_inputDevices[esi]`): +0x4EC the gauge (0 .. 0xFFFF), +0x4F0 the
//! stage (-1 waiting for the button to be let go, 0 running, 1 done, 2 a developer's
//! press), +0x4F4 the start's clock, then the time.
//! - eax 0 starts it: the clock (`system::tick_count`) into +0x4F4, +0x4EC 0, +0x4F0 -1,
//!   `race::draw_tyre_change`; then as -1.
//! - -1: the button read; let go, +0x4F0 0. Result 0.
//! - 0: the button read; pressed: on the developer's machine (`g_developerMachine` 1)
//!   +0x4F0 2 and as 2, else done. Not pressed: running.
//! - 2: with the gauge at 0xF618 or more running, else done.
//! - running: the clock less the start: 60000 or more, the gauge 0 and done; else the gauge
//!   (that x 0x10000 / 500, low 16 bits). Result 0.
//! - done: the time from 0x10000 less the gauge (`log2(sqrt(s x flt_5BEAC8) + flt_5C2C60)
//!   x flt_5BEAC4 x flt_5BEACC / flt_5BEACC + flt_5BEAC0`, x 1000 / 10, `fistp`) into
//!   +0x4F4; below `g_tyreChangeRecord` (unsigned) it is the record, with the player's
//!   name (30 bytes of `g_playerNames`, or of 0x7557A4 for player 1) into
//!   `g_tyreChangeRecordName`; +0x4F0 1, `race::draw_tyre_change`. Result the time.
//! - 1: result the time (+0x4F4).
//!
//! The result also goes into `g_soundScratch` (which the original uses as its scratch).
//!
//! In:  eax 0 to start, else to run; esi the player (`dword_5BEBEC`)
//! Out: eax the result; the rest kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::tyre_change` (twice), which calls the Rust version directly. Its calls
//! by place: 0 `system::tick_count`, 1 `race::draw_tyre_change`, 2 `input_device_read`
//! (0x40930D: ecx, edx, eax, ebx the device's +0x4DC .. +0x4E8; inlined).
//! Refuses (before anything) a player past 7 or a null device.
//! Shadow mode runs the original alone (it reads the clock and the buttons and keeps its
//! own state). Self-test: a copy whose calls go to stand-ins (a scripted clock, the draw
//! noted, the device read from the test's device), against the Rust version with the same:
//! each stage, the button, the developer's machine, the timeout, records beaten and not,
//! both players.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::math::x87::x87;
use crate::race::frame::Part;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::tyre_change_gauge", rva(0x41C1D6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const TICK: u32 = 0;
pub const DRAW: u32 = 1;
pub const READ: u32 = 2;

const DEVICES: usize = 0x4F6EE8; // g_inputDevices
const PLAYER: usize = 0x5BEBEC; // dword_5BEBEC
const RECORD: usize = 0x5BEC28; // g_tyreChangeRecord
const RECORD_NAME: usize = 0x5BEC08; // g_tyreChangeRecordName (30 bytes)
const NAMES: [usize; 2] = [0x755784, 0x7557A4]; // g_playerNames, byte_7557A4
const DEVELOPER: usize = 0x529D94; // g_developerMachine
const SCRATCH: usize = 0x5183CC; // g_soundScratch
/// flt_5BEAC0, flt_5BEAC4, flt_5BEAC8, flt_5BEACC, flt_5C2C60.
const FLOATS: [usize; 5] = [0x5BEAC0, 0x5BEAC4, 0x5BEAC8, 0x5BEACC, 0x5C2C60];
const GAUGE: u32 = 0x4EC;
const STAGE: u32 = 0x4F0;
const CLOCK: u32 = 0x4F4;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// 0x41C2C7 .. 0x41C320: the time of a press, from s = 0x10000 less the gauge.
pub fn press_time(s: i32) -> i32 {
    let f = FLOATS.map(g);
    let v = [
        f[1],
        f[3],
        s as u32,
        f[2],
        f[4],
        f[0],
        0x447A_0000,
        0x4120_0000,
    ];
    let mut out = 0i32;
    x87!(
        "fld dword ptr [{v}]",
        "fmul dword ptr [{v}+4]",
        "fild dword ptr [{v}+8]",
        "fmul dword ptr [{v}+12]",
        "fsqrt",
        "fadd dword ptr [{v}+16]",
        "fyl2x",
        "fdiv dword ptr [{v}+4]",
        "fadd dword ptr [{v}+20]",
        "fmul dword ptr [{v}+24]",
        "fdiv dword ptr [{v}+28]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

#[derive(Clone, Copy)]
enum At {
    Start,
    LetGo,
    Stage,
    Read,
    Run,
    Pressed,
    Developer,
    Done,
    Time,
    Nothing,
}

/// The steps on `r` (eax the result; the rest as at entry).
pub fn tyre_change_gauge_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let entry = *r;
    if entry.esi > 7 {
        return Err(BadInput("a tyre change player past 7").into());
    }
    let d = g(DEVICES + 4 * entry.esi as usize);
    if d == 0 {
        return Err(BadInput("a tyre change player without a device").into());
    }
    p(PLAYER, entry.esi);
    let mut w = Regs {
        esi: d,
        edi: entry.eax,
        ..entry
    };
    let mut at = if entry.eax == 0 { At::Start } else { At::Stage };
    let result = unsafe {
        loop {
            at = match at {
                At::Start => {
                    part(TICK, &mut w)?;
                    put(d + GAUGE, 0);
                    put(d + STAGE, u32::MAX);
                    put(d + CLOCK, w.eax);
                    part(DRAW, &mut w)?;
                    At::LetGo
                }
                At::LetGo => {
                    part(READ, &mut w)?;
                    if w.ebx & 0xFF == 0 {
                        put(d + STAGE, 0);
                    }
                    At::Nothing
                }
                At::Stage => match get(d + STAGE) {
                    u32::MAX => At::LetGo,
                    1 => At::Time,
                    0 => At::Read,
                    _ => At::Developer,
                },
                At::Read => {
                    part(READ, &mut w)?;
                    if w.ebx & 0xFF != 0 {
                        At::Pressed
                    } else {
                        At::Run
                    }
                }
                At::Run => {
                    part(TICK, &mut w)?;
                    w.eax = w.eax.wrapping_sub(get(d + CLOCK));
                    if w.eax >= 60000 {
                        put(d + GAUGE, 0);
                        At::Done
                    } else {
                        let product = w.eax as u64 * 0x1_0000;
                        w.eax = ((product / 500) as u32) & 0xFFFF;
                        w.edx = (product % 500) as u32;
                        w.ebx = 500;
                        put(d + GAUGE, w.eax);
                        At::Nothing
                    }
                }
                At::Pressed => {
                    if g(DEVELOPER) == 1 {
                        put(d + STAGE, 2);
                        At::Developer
                    } else {
                        At::Done
                    }
                }
                At::Developer => {
                    if get(d + GAUGE) >= 0xF618 {
                        At::Run
                    } else {
                        At::Done
                    }
                }
                At::Done => {
                    part(TICK, &mut w)?;
                    w.eax = w.eax.wrapping_sub(get(d + CLOCK));
                    w.ebx = 0x1_0000u32.wrapping_sub(get(d + GAUGE));
                    put(d + CLOCK, press_time(w.ebx as i32) as u32);
                    w.eax = get(d + CLOCK);
                    if w.eax < g(RECORD) {
                        p(RECORD, w.eax);
                        let from = NAMES[(g(PLAYER) != 0) as usize];
                        w.eax = game_addr(rva(from)) as u32;
                        std::ptr::copy(
                            w.eax as *const u8,
                            game_addr(rva(RECORD_NAME)) as *mut u8,
                            30,
                        );
                    }
                    put(d + STAGE, 1);
                    part(DRAW, &mut w)?;
                    At::Time
                }
                At::Time => break get(d + CLOCK),
                At::Nothing => break 0,
            }
        }
    };
    p(SCRATCH, result);
    r.eax = result;
    Ok(())
}

/// `input_device_read` (0x40930D): the device esi's four readings.
fn read_device(r: &mut Regs) {
    unsafe {
        r.ecx = get(r.esi + 0x4DC);
        r.edx = get(r.esi + 0x4E0);
        r.eax = get(r.esi + 0x4E4);
        r.ebx = get(r.esi + 0x4E8);
    }
}

/// The calls in the game: the Rust versions.
pub fn gauge_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        TICK => r.eax = crate::system::tick_count::tick_count(),
        DRAW => unsafe {
            crate::race::draw_tyre_change::draw_tyre_change_in(
                r,
                &mut crate::race::draw_tyre_change::draw_part,
            )?
        },
        _ => read_device(r),
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let mut w = *r;
    match tyre_change_gauge_in(&mut w, &mut gauge_part) {
        Ok(()) => r.eax = w.eax,
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

static CALLED: Mutex<Vec<(u32, [u32; 7])>> = Mutex::new(Vec::new());
/// What the clock's stand-in gives.
static CLOCK_GIVES: AtomicU32 = AtomicU32::new(0);

fn handler(place: u32, r: &mut Regs) {
    CALLED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]));
    match place {
        TICK => r.eax = CLOCK_GIVES.load(Ordering::SeqCst),
        READ => read_device(r),
        _ => {}
    }
}

fn take_called() -> Vec<(u32, [u32; 7])> {
    std::mem::take(&mut *CALLED.lock().unwrap())
}

const PLACES: [(usize, u32); 3] = [(0x411024, TICK), (0x41C38C, DRAW), (0x40930D, READ)];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    set_handler(handler);
    let copy = match unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x41C1D6)),
            0x1B6,
            &[0x60, 0x89, 0x35, 0xEC, 0xEB, 0x5B, 0x00],
            &[0x14, 0x33, 0x38, 0x70, 0x7D, 0xD8, 0x193],
            &|c| {
                PLACES
                    .iter()
                    .find(|&&(k, _)| rva(k) == c)
                    .map(|&(_, n)| stand_in(n))
            },
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut device = vec![0u32; 0x500 / 4];
    let dev = device.as_mut_ptr() as u32;
    let a = |x: usize, len: usize| (game_addr(rva(x)) as u32, len);
    let written: Vec<Region> = vec![
        (dev, 0x500),
        a(PLAYER, 4),
        a(RECORD, 4),
        a(RECORD_NAME, 30),
        a(SCRATCH, 4),
    ];
    let mut kept = written.clone();
    kept.extend([a(DEVICES, 0x20), a(DEVELOPER, 4)]);
    let saved = capture(&kept);
    // (eax, player, stage, button, developer, gauge, elapsed, record)
    let mut cases = vec![];
    for player in [0u32, 1] {
        cases.push((0u32, player, 1u32, 0u32, 0u32, 0u32, 0u32, 0xFFFF_FFFFu32));
        cases.push((0, player, 1, 1, 0, 0, 0, 0xFFFF_FFFF));
        for stage in [u32::MAX, 0, 1, 2, 7] {
            for button in [0u32, 0x100, 1] {
                for developer in [0u32, 1] {
                    for (gauge, elapsed, record) in [
                        (0x1234u32, 250u32, 0xFFFF_FFFFu32),
                        (0xF700, 499, 50),
                        (0x8000, 60000, 0xFFFF_FFFF),
                        (0xFFFF, 1000, 0),
                    ] {
                        cases.push((1, player, stage, button, developer, gauge, elapsed, record));
                    }
                }
            }
        }
    }
    for (eax, player, stage, button, developer, gauge, elapsed, record) in cases {
        let fill = || unsafe {
            restore(&kept, &saved);
            for k in 0..0x140u32 {
                put(dev + 4 * k, 0x0101_0000 + k);
            }
            put(dev + 0x4E8, button);
            put(dev + GAUGE, gauge);
            put(dev + STAGE, stage);
            put(dev + CLOCK, 100_000);
            game_write(rva(DEVICES) + 4 * player as usize, dev);
            p(DEVELOPER, developer);
            p(RECORD, record);
            CLOCK_GIVES.store(100_000 + elapsed, Ordering::SeqCst);
        };
        let entry = Regs {
            eax,
            esi: player,
            ..Regs::SENTINEL
        };
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        fill();
        take_called();
        let mut ro = entry;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&written));
        fill();
        let mut rr = entry;
        let ok = tyre_change_gauge_in(&mut rr, &mut |p, r| {
            handler(p, r);
            Ok(())
        })
        .is_ok();
        let mine = (ok.then(|| regs(&rr)), take_called(), capture(&written));
        t.check(theirs == mine, || {
            let diff = theirs.2.iter().zip(&mine.2).position(|(x, y)| x != y);
            format!(
                "eax {eax}, player {player}, stage {stage:#x}, button {button:#x}, developer \
                 {developer}, gauge {gauge:#x}, elapsed {elapsed}, record {record:#x}: original \
                 {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
                theirs.0, theirs.1, mine.0, mine.1
            )
        });
    }
    restore(&kept, &saved);
    drop((copy, device));
    t.finish()
}
