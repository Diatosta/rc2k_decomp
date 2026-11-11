//! The roadside tyre change's gauge (from `race::tyre_change_gauge`
//! 0x41C1D6): a gauge filling over half a second while a button is held, the press
//! timing the change; the player's device keeps the gauge, the stage and the start's
//! clock (then the time). The clock, the button, the drawing and the press's time (the
//! game's float formula) come through `Io`, called in the original's order.

/// The gauge's stage (the device's +0x4F0).
pub mod stage {
    /// Waiting for the button to be let go.
    pub const LET_GO: u32 = u32::MAX;
    /// Running.
    pub const RUNNING: u32 = 0;
    /// Done: the time is kept.
    pub const DONE: u32 = 1;
    /// A developer's press: running on to the gauge's top.
    pub const DEVELOPER: u32 = 2;
}

/// The longest a change runs (ms) before it is done with an empty gauge.
pub const LONGEST: u32 = 60000;
/// The time (ms) the gauge takes to fill.
pub const FILL_MS: u32 = 500;
/// Where a developer's press stops.
pub const DEVELOPER_TOP: u32 = 0xF618;

/// The gauge's state (the device's +0x4EC, +0x4F0, +0x4F4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gauge {
    pub gauge: u32,
    pub stage: u32,
    /// The start's clock, then the time.
    pub clock: u32,
}

/// The gauge's dealings with the game, in the original's order.
pub trait Io {
    type Error;
    /// The clock (ms).
    fn tick(&mut self) -> Result<u32, Self::Error>;
    /// The button's state: whether it is pressed.
    fn pressed(&mut self) -> Result<bool, Self::Error>;
    /// The gauge drawn, as it now is.
    fn draw(&mut self, gauge: &Gauge) -> Result<(), Self::Error>;
    /// The time of a press, from 0x10000 less the gauge.
    fn press_time(&mut self, s: u32) -> u32;
    /// Whether this is a developer's machine (asked at a press).
    fn developer(&mut self) -> bool;
    /// The record time (asked once a press is timed).
    fn record(&mut self) -> u32;
    /// The running gauge worked out: the time since the start, x 0x10000, over
    /// `FILL_MS`: the gauge (its low 16 bits) and the rest.
    fn ran(&mut self, _gauge: u32, _rest: u32) {}
    /// The press timed: the time since the start and s.
    fn timed(&mut self, _elapsed: u32, _s: u32) {}
    /// The gauge ran out: `elapsed`, the time it ran (the original keeps it in a register).
    fn timed_out(&mut self, _elapsed: u32) {}
    /// The record beaten by `time` (unsigned): the new record, the player's name to
    /// copy.
    fn beaten(&mut self, time: u32);
}

/// The gauge's next turn: `start` (mode 0) starts it (the clock, the gauge 0, waiting for
/// the button to be let go, drawn), then as waiting; waiting, the button let go starts
/// it running; running, a press ends it (on the developer's machine it runs on to
/// `DEVELOPER_TOP`), else the gauge follows the clock (`LONGEST` or more: an empty gauge,
/// done); done, the time from the press (the record beaten when below it, unsigned),
/// drawn. Gives the result: the time once done, else 0.
pub fn turn<I: Io>(start: bool, g: &mut Gauge, io: &mut I) -> Result<u32, I::Error> {
    enum At {
        LetGo,
        Read,
        Run,
        Developer,
        Done,
    }
    let mut at = if start {
        let clock = io.tick()?;
        *g = Gauge {
            gauge: 0,
            stage: stage::LET_GO,
            clock,
        };
        io.draw(g)?;
        At::LetGo
    } else {
        match g.stage {
            stage::LET_GO => At::LetGo,
            stage::DONE => return Ok(g.clock),
            stage::RUNNING => At::Read,
            _ => At::Developer,
        }
    };
    loop {
        at = match at {
            At::LetGo => {
                if !io.pressed()? {
                    g.stage = stage::RUNNING;
                }
                return Ok(0);
            }
            At::Read => {
                if !io.pressed()? {
                    At::Run
                } else if io.developer() {
                    g.stage = stage::DEVELOPER;
                    At::Developer
                } else {
                    At::Done
                }
            }
            At::Developer => {
                if g.gauge >= DEVELOPER_TOP {
                    At::Run
                } else {
                    At::Done
                }
            }
            At::Run => {
                let elapsed = io.tick()?.wrapping_sub(g.clock);
                if elapsed >= LONGEST {
                    io.timed_out(elapsed);
                    g.gauge = 0;
                    At::Done
                } else {
                    let product = elapsed as u64 * 0x1_0000;
                    g.gauge = ((product / FILL_MS as u64) as u32) & 0xFFFF;
                    io.ran(g.gauge, (product % FILL_MS as u64) as u32);
                    return Ok(0);
                }
            }
            At::Done => {
                let elapsed = io.tick()?.wrapping_sub(g.clock);
                let s = 0x1_0000u32.wrapping_sub(g.gauge);
                io.timed(elapsed, s);
                g.clock = io.press_time(s);
                if g.clock < io.record() {
                    io.beaten(g.clock);
                }
                g.stage = stage::DONE;
                io.draw(g)?;
                return Ok(g.clock);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Script {
        clock: Vec<u32>,
        pressed: Vec<bool>,
        drawn: Vec<Gauge>,
        beaten: Option<u32>,
        developer: bool,
        record: u32,
    }

    impl Io for Script {
        type Error = ();
        fn tick(&mut self) -> Result<u32, ()> {
            Ok(self.clock.remove(0))
        }
        fn pressed(&mut self) -> Result<bool, ()> {
            Ok(self.pressed.remove(0))
        }
        fn draw(&mut self, g: &Gauge) -> Result<(), ()> {
            self.drawn.push(*g);
            Ok(())
        }
        fn press_time(&mut self, s: u32) -> u32 {
            s / 16
        }
        fn developer(&mut self) -> bool {
            self.developer
        }
        fn record(&mut self) -> u32 {
            self.record
        }
        fn beaten(&mut self, time: u32) {
            self.beaten = Some(time);
        }
    }

    fn script(clock: &[u32], pressed: &[bool], developer: bool, record: u32) -> Script {
        Script {
            clock: clock.to_vec(),
            pressed: pressed.to_vec(),
            drawn: Vec::new(),
            beaten: None,
            developer,
            record,
        }
    }

    #[test]
    fn a_change() {
        let mut g = Gauge {
            gauge: 9,
            stage: 7,
            clock: 9,
        };
        // Started with the button still held: waiting.
        let mut io = script(&[1000], &[true], false, 0);
        assert_eq!(turn(true, &mut g, &mut io), Ok(0));
        assert_eq!((g.stage, io.drawn.len()), (stage::LET_GO, 1));
        // Let go: running.
        let mut io = script(&[], &[false], false, 0);
        assert_eq!(turn(false, &mut g, &mut io), Ok(0));
        assert_eq!(g.stage, stage::RUNNING);
        // A quarter of a second in: the gauge half full.
        let mut io = script(&[1250], &[false], false, 0);
        assert_eq!(turn(false, &mut g, &mut io), Ok(0));
        assert_eq!(g.gauge, 0x8000);
        // The press: timed from what the gauge left, the record beaten.
        let mut io = script(&[1300], &[true], false, 0x1000);
        assert_eq!(turn(false, &mut g, &mut io), Ok(0x800));
        assert_eq!(
            (g.stage, g.clock, io.beaten),
            (stage::DONE, 0x800, Some(0x800))
        );
        // Done: the time again.
        let mut io = script(&[], &[], false, 0);
        assert_eq!(turn(false, &mut g, &mut io), Ok(0x800));
    }

    #[test]
    fn developer_and_timeout() {
        let mut g = Gauge {
            gauge: 0x100,
            stage: stage::RUNNING,
            clock: 0,
        };
        let mut io = script(&[60000], &[true], true, 0);
        assert_eq!(turn(false, &mut g, &mut io), Ok(0xFF00 / 16));
        assert_eq!(g.stage, stage::DONE);
        let mut g = Gauge {
            gauge: DEVELOPER_TOP,
            stage: stage::DEVELOPER,
            clock: 0,
        };
        let mut io = script(&[60000, 60000], &[], true, 0);
        assert_eq!(turn(false, &mut g, &mut io), Ok(0x1_0000 / 16));
        assert_eq!(g.gauge, 0);
    }
}
