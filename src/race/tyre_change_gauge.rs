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
use rc2k_core::race::tyre_change::{FILL_MS, Gauge, Io, turn};
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

/// The gauge's dealings with the game: the parts on the working registers (the device in
/// esi), its state written into the device before each drawing.
struct GameIo<'a> {
    w: Regs,
    device: u32,
    part: Part<'a>,
}

impl GameIo<'_> {
    fn store(&self, g: &Gauge) {
        unsafe {
            put(self.device + GAUGE, g.gauge);
            put(self.device + STAGE, g.stage);
            put(self.device + CLOCK, g.clock);
        }
    }
}

impl Io for GameIo<'_> {
    type Error = Refused;
    fn tick(&mut self) -> Result<u32, Refused> {
        (self.part)(TICK, &mut self.w)?;
        Ok(self.w.eax)
    }
    fn pressed(&mut self) -> Result<bool, Refused> {
        (self.part)(READ, &mut self.w)?;
        Ok(self.w.ebx & 0xFF != 0)
    }
    fn draw(&mut self, g: &Gauge) -> Result<(), Refused> {
        self.store(g);
        (self.part)(DRAW, &mut self.w)
    }
    fn press_time(&mut self, s: u32) -> u32 {
        // eax the time as stored.
        self.w.eax = press_time(s as i32) as u32;
        self.w.eax
    }
    fn developer(&mut self) -> bool {
        g(DEVELOPER) == 1
    }
    fn record(&mut self) -> u32 {
        g(RECORD)
    }
    fn ran(&mut self, gauge: u32, rest: u32) {
        (self.w.eax, self.w.edx, self.w.ebx) = (gauge, rest, FILL_MS);
    }
    fn timed_out(&mut self, elapsed: u32) {
        self.w.eax = elapsed;
    }
    fn timed(&mut self, elapsed: u32, s: u32) {
        (self.w.eax, self.w.ebx) = (elapsed, s);
    }
    fn beaten(&mut self, time: u32) {
        p(RECORD, time);
        let from = NAMES[(g(PLAYER) != 0) as usize];
        self.w.eax = game_addr(rva(from)) as u32;
        unsafe {
            std::ptr::copy(
                self.w.eax as *const u8,
                game_addr(rva(RECORD_NAME)) as *mut u8,
                30,
            )
        };
    }
}

/// The steps on `r` (eax the result; the rest as at entry).
pub fn tyre_change_gauge_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::tyre_change::turn`.
    let entry = *r;
    if entry.esi > 7 {
        return Err(BadInput("a tyre change player past 7").into());
    }
    let d = g(DEVICES + 4 * entry.esi as usize);
    if d == 0 {
        return Err(BadInput("a tyre change player without a device").into());
    }
    p(PLAYER, entry.esi);
    let mut io = GameIo {
        w: Regs {
            esi: d,
            edi: entry.eax,
            ..entry
        },
        device: d,
        part,
    };
    let mut gauge = unsafe {
        Gauge {
            gauge: get(d + GAUGE),
            stage: get(d + STAGE),
            clock: get(d + CLOCK),
        }
    };
    let before = gauge;
    let result = turn(entry.eax == 0, &mut gauge, &mut io)?;
    if gauge != before {
        io.store(&gauge);
    }
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
