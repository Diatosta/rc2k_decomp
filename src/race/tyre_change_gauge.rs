//! `race::tyre_change_gauge`: the roadside tyre change's gauge, timed by a button (the low byte
//! of the input device's `bits`). The state is the device's (`input_devices[player]`): `gauge`
//! (0 .. 0xFFFF), `stage` (-1 waiting for the button to be let go, 0 running, 1 done) and `clock` (the start's clock, then the time).
//! - Mode 0 starts it: the clock (`clock::tick_count`) goes into `clock`, `gauge` is 0, `stage`
//!   -1 and `race::draw_tyre_change` draws; then as -1.
//! - -1: the button is read; let go, `stage` becomes 0. Result 0.
//! - 0: the button is read; pressed: it is done. Not pressed: running.
//! - running: if the clock less the start is 60000 or more, the gauge is 0 and it is done; else
//!   the gauge is that x 0x10000 / 500, low 16 bits. Result 0.
//! - done: the time is worked out from 0x10000 less the gauge (a logarithm of the square root of
//!   the gauge times `gauge_unit`, scaled with `time_scale`, `log_two_ten` and `time_base`, x 1000
//!   / 10, rounded) and goes into `clock`. Below `TyreChange::record` (unsigned) it is the new
//!   record, with the player's name (30 bytes of `player_names`, or of the second player's for
//!   player 1) going into `TyreChange::record_name`; `stage` becomes 1 and
//!   `race::draw_tyre_change` draws. Result the time.
//! - 1: result the time (`clock`).
//!
//! The result also goes into the machine's scratch dword.
//!
//! Refuses (before anything) a player past 7 or a missing device.

use crate::errors::{BadInput, Refused};
use crate::input::types::InputDevice;
use crate::owners::Owners;
use crate::race::owner::Race;
use rc2k_core::race::tyre_change::{Gauge, Io, turn};

/// The press time's constants: `TyreChange::{time_base, time_scale, gauge_unit, log_two_ten}` and
/// `one`.
fn constants(race: &Race) -> [f32; 5] {
    let shared = &race.shared;
    [
        shared.tyre_change().time_base.get(),
        shared.tyre_change().time_scale.get(),
        shared.tyre_change().gauge_unit.get(),
        shared.tyre_change().log_two_ten.get(),
        race.viewer_panel().one.get(),
    ]
}

/// The time of a press, from s = 0x10000 less the gauge.
pub fn press_time(race: &Race, s: i32) -> i32 {
    use crate::math::x87_float::{X, int, ld, to_int};
    let f = constants(race).map(ld);
    // The product and the sum are the x87's; the logarithm is libm's log2.
    let log = (int(s) * f[2]).sqrt() + f[4];
    let r = f[1] * f[3] * X(rc2k_core::math::portable::log2(log.0));
    let t = (r / f[3] + f[0]) * ld(0x447A_0000) / ld(0x4120_0000);
    to_int(t)
}

/// The gauge's connections to the rest of the game: the device's state is written into it before
/// each drawing.
struct GameIo {
    o: Owners<'static>,
    race: &'static Race,
    device: &'static InputDevice,
}

impl GameIo {
    fn store(&self, g: &Gauge) {
        self.device.gauge.set(g.gauge);
        self.device.stage.set(g.stage);
        self.device.clock.set(g.clock);
    }
}

impl Io for GameIo {
    type Error = Refused;
    fn tick(&mut self) -> Result<u32, Refused> {
        Ok(crate::clock::tick_count::tick_count(
            &self.o.system.session.picture_clock,
        ))
    }
    fn pressed(&mut self) -> Result<bool, Refused> {
        // The button is the low byte of the device's `bits`.
        Ok(self.device.bits.get() & 0xFF != 0)
    }
    fn draw(&mut self, g: &Gauge) -> Result<(), Refused> {
        let race = self.race;
        let shared = &race.shared;
        self.store(g);
        // The drawing takes the device itself (its x87 turn reads the gauge).
        crate::race::draw_tyre_change::draw_tyre_change(self.o, race, shared, self.device)
    }
    fn press_time(&mut self, s: u32) -> u32 {
        press_time(self.race, s as i32) as u32
    }
    fn record(&mut self) -> u32 {
        self.race.shared.tyre_change().record.get()
    }
    fn beaten(&mut self, time: u32) {
        let shared = &self.race.shared;
        shared.tyre_change().record.set(time);
        // The 30 bytes of the player's name in `Names.player_names`: the first player's, or the
        // second's for player 1.
        let from = 0x20 * (shared.tyre_change().player.get() != 0) as usize;
        for n in 0..30 {
            shared
                .tyre_change()
                .record_name
                .set(n, self.o.menus.play.slots.names.player_names.get(from + n));
        }
    }
}

/// Starts the gauge when `start` is set, else runs it, for player `player`; gives the result.
pub fn tyre_change_gauge(o: Owners<'static>, start: bool, player: u32) -> Result<u32, Refused> {
    let (race, input, system) = (o.race, o.input, o.system);
    if player > 7 {
        return Err(BadInput("a tyre change player past 7").into());
    }
    let Some(d) = input.devices.input_device(player as usize)? else {
        return Err(BadInput("a tyre change player without a device").into());
    };
    race.shared.tyre_change().player.set(player);
    let mut io = GameIo { o, race, device: d };
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
    system.machine().scratch.set(result);
    Ok(result)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word};
    use crate::race::viewer_screen_store::VIEWER_PANEL;
    use crate::test_support::own::race;
    use crate::test_support::vectors::{Recorder, word};

    /// The five constants.
    fn constants() -> [f32; 5] {
        let tyre = race().shared.tyre_change();
        [
            tyre.time_base.get(),
            tyre.time_scale.get(),
            tyre.gauge_unit.get(),
            tyre.log_two_ten.get(),
            VIEWER_PANEL.one.get(),
        ]
    }

    /// The five constants set.
    fn set_constants(v: [f32; 5]) {
        let tyre = race().shared.tyre_change();
        tyre.time_base.set(v[0]);
        tyre.time_scale.set(v[1]);
        tyre.gauge_unit.set(v[2]);
        tyre.log_two_ten.set(v[3]);
        VIEWER_PANEL.one.set(v[4]);
    }

    /// The five constants as they were, put back when dropped.
    struct Kept([f32; 5]);

    impl Drop for Kept {
        fn drop(&mut self) {
            set_constants(self.0);
        }
    }

    /// With the stored constants, over the whole range of the gauge, then with other constants.
    /// The logarithm is libm's here: a result may differ by one where the x87's log2 and libm's
    /// disagree at the last bit and the sum falls on a whole number; counted.
    #[test]
    fn press_time_matches() {
        let _kept = Kept(constants());
        let mut r = Rng(31);
        let mut file = Recorder::replay("race.tyre_change_gauge.press_time");
        let mut off = 0;
        let mut total = 0;
        at_each_control_word(|| {
            for n in 0..20000u32 {
                if n >= 8000 {
                    set_constants(std::array::from_fn(|_| f32::from_bits(r.float())));
                }
                let s = if n < 8000 {
                    (n * 8 % 0x10000) as i32
                } else {
                    r.int()
                };
                let inputs = constants().map(f32::to_bits);
                for w in inputs.iter().chain(&[s as u32]) {
                    file.input(&w.to_le_bytes());
                }
                let (want, got) = (word(&mut file) as i32, press_time(race(), s));
                total += 1;
                if want != got {
                    off += 1;
                    assert!(n >= 8000 || want.abs_diff(got) <= 1, "{s}: {want} {got}");
                }
            }
        });
        file.finish();
        eprintln!("press_time: {off} of {total} differ");
        assert!(off * 1000 < total, "{off} of {total}");
    }
}
