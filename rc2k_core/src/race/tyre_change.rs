//! The roadside tyre change's gauge: it fills over half a second while a button is held and the
//! press times the change. The player's device keeps the gauge, stage and start clock (then the
//! time). Clock, button, drawing and press time come through `Io`.

/// The gauge's stage.
pub mod stage {
    /// Waiting for the button to be let go.
    pub const LET_GO: u32 = u32::MAX;
    /// Running.
    pub const RUNNING: u32 = 0;
    /// Done: the time is kept.
    pub const DONE: u32 = 1;
}

/// Longest a change runs (ms) before it ends with an empty gauge.
pub const LONGEST: u32 = 60000;
/// The time (ms) the gauge takes to fill.
pub(crate) const FILL_MS: u32 = 500;

/// The gauge's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gauge {
    /// The gauge's fill.
    pub gauge: u32,
    /// One of the `stage` constants.
    pub stage: u32,
    /// The start's clock, then the time.
    pub clock: u32,
}

/// The gauge's dealings with the game, called in a fixed order.
pub trait Io {
    type Error;
    /// The clock (ms).
    fn tick(&mut self) -> Result<u32, Self::Error>;
    /// Whether the button is pressed.
    fn pressed(&mut self) -> Result<bool, Self::Error>;
    /// Draws the gauge as it now is.
    fn draw(&mut self, gauge: &Gauge) -> Result<(), Self::Error>;
    /// The time of a press, from 0x10000 less the gauge.
    fn press_time(&mut self, s: u32) -> u32;
    /// The record time (asked once a press is timed).
    fn record(&mut self) -> u32;
    /// Notes the running gauge: time since the start x 0x10000 over `FILL_MS` gives the gauge (low
    /// 16 bits) and the rest.
    fn ran(&mut self, _gauge: u32, _rest: u32) {}
    /// The press timed: the time since the start and `s`.
    fn timed(&mut self, _elapsed: u32, _s: u32) {}
    /// The gauge ran out after `elapsed` ms.
    fn timed_out(&mut self, _elapsed: u32) {}
    /// The record beaten by `time` (unsigned): the new record, the player's name to copy.
    fn beaten(&mut self, time: u32);
}

/// The gauge's next turn. `start` (mode 0) starts it (clock, gauge 0, waiting for the button
/// to be let go, drawn). Waiting: letting go starts it running. Running: a press ends it, else the gauge follows the clock
/// (`LONGEST` or more: empty gauge, done). Done: the time from the press (record beaten when
/// below it, unsigned), drawn. Returns the time once done, else 0.
pub fn turn<I: Io>(start: bool, g: &mut Gauge, io: &mut I) -> Result<u32, I::Error> {
    enum At {
        LetGo,
        Read,
        Run,
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
            _ => At::Read,
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
        fn record(&mut self) -> u32 {
            self.record
        }
        fn beaten(&mut self, time: u32) {
            self.beaten = Some(time);
        }
    }

    fn script(clock: &[u32], pressed: &[bool], record: u32) -> Script {
        Script {
            clock: clock.to_vec(),
            pressed: pressed.to_vec(),
            drawn: Vec::new(),
            beaten: None,
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
        let mut io = script(&[1000], &[true], 0);
        assert_eq!(turn(true, &mut g, &mut io), Ok(0));
        assert_eq!((g.stage, io.drawn.len()), (stage::LET_GO, 1));
        // Let go: running.
        let mut io = script(&[], &[false], 0);
        assert_eq!(turn(false, &mut g, &mut io), Ok(0));
        assert_eq!(g.stage, stage::RUNNING);
        // A quarter of a second in: the gauge half full.
        let mut io = script(&[1250], &[false], 0);
        assert_eq!(turn(false, &mut g, &mut io), Ok(0));
        assert_eq!(g.gauge, 0x8000);
        // The press: timed from what the gauge left, the record beaten.
        let mut io = script(&[1300], &[true], 0x1000);
        assert_eq!(turn(false, &mut g, &mut io), Ok(0x800));
        assert_eq!(
            (g.stage, g.clock, io.beaten),
            (stage::DONE, 0x800, Some(0x800))
        );
        // Done: the time again.
        let mut io = script(&[], &[], 0);
        assert_eq!(turn(false, &mut g, &mut io), Ok(0x800));
    }

    #[test]
    fn press_and_timeout() {
        let mut g = Gauge {
            gauge: 0x100,
            stage: stage::RUNNING,
            clock: 0,
        };
        let mut io = script(&[60000], &[true], 0);
        assert_eq!(turn(false, &mut g, &mut io), Ok(0xFF00 / 16));
        assert_eq!(g.stage, stage::DONE);
        let mut g = Gauge {
            gauge: 0xF618,
            stage: stage::RUNNING,
            clock: 0,
        };
        let mut io = script(&[60000, 60000], &[false], 0);
        assert_eq!(turn(false, &mut g, &mut io), Ok(0x1_0000 / 16));
        assert_eq!(g.gauge, 0);
    }
}
