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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87::x87;
use crate::race::frame::Part;
use rc2k_core::race::tyre_change::{FILL_MS, Gauge, Io, turn};

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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let mut w = *r;
    match tyre_change_gauge_in(&mut w, &mut gauge_part) {
        Ok(()) => r.eax = w.eax,
        Err(e) => return Err(e),
    }
    Ok(())
}
