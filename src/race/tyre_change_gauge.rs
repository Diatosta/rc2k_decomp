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
//! Caller: `race::tyre_change` (twice), which calls the Rust version directly and reads only
//! eax. Its calls: `system::tick_count`, `race::draw_tyre_change`, `input_device_read`
//! (0x40930D: ecx, edx, eax, ebx the device's +0x4DC .. +0x4E8; inlined). Refuses (before
//! anything) a player past 7 or a null device.

use crate::errors::{BadInput, Refused};
use crate::input::owner::input;
use crate::input::types::InputDevice;
use crate::menu::owner::menus;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::system::owner::system;
use rc2k_core::race::tyre_change::{Gauge, Io, turn};

/// flt_5C2C60 (`VIEWER_PANEL.one`): added to the square root in the press time's logarithm.
#[cfg(all(test, target_arch = "x86"))]
const SQRT_OFFSET: usize = 0x5C2C60;

#[cfg(all(test, target_arch = "x86"))]
fn p(x: usize, v: u32) {
    crate::store::game_write(crate::framework::rva(x), v);
}

/// The press time's constants: `TyreChange::{time_base, time_scale, gauge_unit,
/// log_two_ten}` and flt_5C2C60.
fn constants(shared: &Shared) -> [u32; 5] {
    [
        shared.tyre_change.time_base.bits(),
        shared.tyre_change.time_scale.bits(),
        shared.tyre_change.gauge_unit.bits(),
        shared.tyre_change.log_two_ten.bits(),
        VIEWER_PANEL.one.bits(),
    ]
}

/// 0x41C2C7 .. 0x41C320: the time of a press, from s = 0x10000 less the gauge.
pub fn press_time(shared: &Shared, s: i32) -> i32 {
    use crate::math::x87_float::{X, int, ld, to_int};
    let f = constants(shared).map(ld);
    // `fyl2x` (libm's log2): the product and the sum are the x87's
    let log = (int(s) * f[2]).sqrt() + f[4];
    let r = f[1] * f[3] * X(log.0.log2());
    let t = (r / f[3] + f[0]) * ld(0x447A_0000) / ld(0x4120_0000);
    to_int(t)
}

/// The gauge's dealings with the game: the device's state written into it before each
/// drawing.
struct GameIo<'a> {
    race: &'a Race,
    device: &'static InputDevice,
}

impl GameIo<'_> {
    fn store(&self, g: &Gauge) {
        self.device.gauge.set(g.gauge);
        self.device.stage.set(g.stage);
        self.device.clock.set(g.clock);
    }
}

impl Io for GameIo<'_> {
    type Error = Refused;
    fn tick(&mut self) -> Result<u32, Refused> {
        Ok(crate::system::tick_count::tick_count())
    }
    fn pressed(&mut self) -> Result<bool, Refused> {
        // `input_device_read` (0x40930D, inlined): ebx the device's +0x4E8.
        Ok(self.device.bits.get() & 0xFF != 0)
    }
    fn draw(&mut self, g: &Gauge) -> Result<(), Refused> {
        // `draw_tyre_change` wants the `'static` owner.
        let race = crate::race::owner::race();
        let shared = &race.shared;
        self.store(g);
        // The drawing takes the device by address (its x87 turn reads the gauge).
        crate::race::draw_tyre_change::draw_tyre_change(race, shared, self.device)
    }
    fn press_time(&mut self, s: u32) -> u32 {
        let shared = &self.race.shared;
        press_time(shared, s as i32) as u32
    }
    fn developer(&mut self) -> bool {
        crate::system::check_developer_machine::DEVELOPER.get() == 1
    }
    fn record(&mut self) -> u32 {
        self.race.shared.tyre_change.record.get()
    }
    fn beaten(&mut self, time: u32) {
        let shared = &self.race.shared;
        shared.tyre_change.record.set(time);
        // g_playerNames, or the second name, byte_7557A4: 30 bytes.
        let from = 0x20 * (shared.tyre_change.player.get() != 0) as usize;
        for n in 0..30 {
            shared
                .tyre_change
                .record_name
                .set(n, menus().play.names.player_names.get(from + n));
        }
    }
}

/// The Rust version: `start` (eax 0) starts the gauge, else runs it, for player `player`
/// (esi); gives the result (eax; the original keeps the rest).
pub fn tyre_change_gauge(race: &Race, start: bool, player: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::race::tyre_change::turn`.
    if player > 7 {
        return Err(BadInput("a tyre change player past 7").into());
    }
    let Some(d) = input().devices.input_device(player as usize)? else {
        return Err(BadInput("a tyre change player without a device").into());
    };
    race.shared.tyre_change.player.set(player);
    let mut io = GameIo { race, device: d };
    let mut gauge = Gauge {
        gauge: d.gauge.get(),
        stage: d.stage.get(),
        clock: d.clock.get(),
    };
    let before = gauge;
    let result = turn(start, &mut gauge, &mut io)?;
    if gauge != before {
        io.store(&gauge);
    }
    system().session.machine.scratch.set(result);
    Ok(result)
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::math::x87::x87;
    use crate::race::asm_test::{Rng, Saved, at_each_control_word, lock};
    use crate::race::owner::race;

    /// The four constants as they were, put back when dropped.
    struct Kept([u32; 4]);

    impl Drop for Kept {
        fn drop(&mut self) {
            let shared = &race().shared;
            shared.tyre_change.time_base.set_bits(self.0[0]);
            shared.tyre_change.time_scale.set_bits(self.0[1]);
            shared.tyre_change.gauge_unit.set_bits(self.0[2]);
            shared.tyre_change.log_two_ten.set_bits(self.0[3]);
        }
    }

    // The original block, verbatim.
    pub(super) fn press_time_asm(s: i32) -> i32 {
        let shared = &crate::race::owner::race().shared;
        let f = constants(shared);
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

    /// With the game's own constants, over the whole range of the gauge, then with other
    /// constants. `fyl2x` is libm's here: a result may differ by one where the x87's log2
    /// and libm's disagree at the last bit and the sum falls on a whole number; counted.
    #[test]
    fn press_time_matches() {
        let shared = &race().shared;
        let _l = lock();
        let _saved = Saved::new(&[SQRT_OFFSET]);
        let _kept = Kept([
            shared.tyre_change.time_base.bits(),
            shared.tyre_change.time_scale.bits(),
            shared.tyre_change.gauge_unit.bits(),
            shared.tyre_change.log_two_ten.bits(),
        ]);
        let mut r = Rng(31);
        let mut off = 0;
        let mut total = 0;
        at_each_control_word(|| {
            for n in 0..60000u32 {
                if n >= 20000 {
                    shared.tyre_change.time_base.set_bits(r.float());
                    shared.tyre_change.time_scale.set_bits(r.float());
                    shared.tyre_change.gauge_unit.set_bits(r.float());
                    shared.tyre_change.log_two_ten.set_bits(r.float());
                    p(SQRT_OFFSET, r.float());
                }
                let s = if n < 20000 {
                    (n % 0x10000) as i32
                } else {
                    r.int()
                };
                let (want, got) = (press_time_asm(s), press_time(shared, s));
                total += 1;
                if want != got {
                    off += 1;
                    assert!(n >= 20000 || want.abs_diff(got) <= 1, "{s}: {want} {got}");
                }
            }
        });
        eprintln!("press_time: {off} of {total} differ");
        assert!(off * 1000 < total, "{off} of {total}");
    }
}
