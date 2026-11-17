//! The race's reverb: a small Schroeder / Freeverb network (eight damped feedback combs in
//! parallel, four all-passes in series, a second set a few samples longer for the right ear)
//! fed one mono send and giving a stereo tail.
//!
//! The rooms are the ten presets the original asked the EAX 1 sound cards for (the reverb
//! environment, volume, decay time and damping of each). EAX 1 gave a card's own reverb only
//! those three numbers to work with, so they are what shapes this one too: the decay time sets
//! every comb's feedback so that the tail falls by 60 dB in that time, the damping how fast the
//! high frequencies die in the loop, and the volume the level of the tail beside the dry sound.
//! The environment number only named which of the card's built-in rooms the three numbers
//! started from; nothing here reads it.

/// A room preset: the four values of an EAX 1 reverb as the original's table holds them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Room {
    /// EAX 1's environment number (kept to show where the numbers come from).
    pub environment: u32,
    /// The tail's level, 0.0 ..= 1.0.
    pub volume: f32,
    /// Seconds for the tail to fall by 60 dB.
    pub decay: f32,
    /// How fast high frequencies die in the tail, 0.0 ..= 1.0.
    pub damping: f32,
}

/// The ten rooms, in the original's table's order (room 3, the one a race asks for, is the
/// forest environment with a short, light tail).
pub const ROOMS: [Room; 10] = [
    Room {
        environment: 2,
        volume: 0.5,
        decay: 0.5,
        damping: 0.5,
    },
    Room {
        environment: 15,
        volume: 0.111,
        decay: 3.145,
        damping: 0.472,
    },
    Room {
        environment: 15,
        volume: 0.25,
        decay: 0.1,
        damping: 0.1,
    },
    Room {
        environment: 15,
        volume: 1.0,
        decay: 0.1,
        damping: 0.1,
    },
    Room {
        environment: 15,
        volume: 0.0,
        decay: 0.1,
        damping: 0.5,
    },
    Room {
        environment: 19,
        volume: 0.097,
        decay: 2.767,
        damping: 0.224,
    },
    Room {
        environment: 16,
        volume: 0.111,
        decay: 2.767,
        damping: 0.224,
    },
    Room {
        environment: 17,
        volume: 0.25,
        decay: 2.0,
        damping: 0.35,
    },
    Room {
        environment: 17,
        volume: 0.25,
        decay: 4.0,
        damping: 0.35,
    },
    Room {
        environment: 18,
        volume: 1.0,
        decay: 1.499,
        damping: 0.5,
    },
];

/// The room the race asks for at its start.
pub const RACE_ROOM: usize = 3;

/// The tail's level at a room volume of 1.0 and the fullest weight (the dry sound is 1.0).
const WET_LEVEL: f32 = 0.45;
/// The weight at which a view hears half the tail: the weight is a sum of the course's
/// reverberant surfaces near the car (a race starts at 1.0).
const WEIGHT_HALF: f32 = 4.0;
/// The share of the tail an open place has: a short, low ambience (the weights measured in open
/// stages run from about 10 to 50, which tells open country from a street only by a little, so
/// the weight alone does not say a place is enclosed).
const OPEN_SHARE: f32 = 0.2;
/// The weights between which the share grows from `OPEN_SHARE` to the whole tail: above the
/// highest of the open stages' weights, where the surroundings are very reverberant (a tunnel).
const ENCLOSED_FROM: f32 = 50.0;
const ENCLOSED_FULL: f32 = 130.0;

/// How much of a sound goes to the reverb, from the room and the weight of the listening
/// view's surroundings (0.0 ..= `WET_LEVEL`).
pub fn send_level(room: &Room, weight: f32) -> f32 {
    if !weight.is_finite() || weight <= 0.0 {
        return 0.0;
    }
    let enclosed = ((weight - ENCLOSED_FROM) / (ENCLOSED_FULL - ENCLOSED_FROM)).clamp(0.0, 1.0);
    let share = OPEN_SHARE + (1.0 - OPEN_SHARE) * enclosed;
    room.volume.clamp(0.0, 1.0) * WET_LEVEL * share * weight / (weight + WEIGHT_HALF)
}

/// Freeverb's tunings, in samples at 44100 Hz.
const COMB_LENGTHS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS_LENGTHS: [usize; 4] = [556, 441, 341, 225];
/// The right ear's lines are this much longer.
const SPREAD: usize = 23;
const ALLPASS_FEEDBACK: f32 = 0.5;
/// The most a comb's feedback may be (a tail never grows).
const MOST_FEEDBACK: f32 = 0.98;

struct Comb {
    line: Vec<f32>,
    at: usize,
    low_pass: f32,
    feedback: f32,
    damping: f32,
}

impl Comb {
    fn new(length: usize) -> Self {
        Comb {
            line: vec![0.0; length.max(1)],
            at: 0,
            low_pass: 0.0,
            feedback: 0.0,
            damping: 0.0,
        }
    }

    fn step(&mut self, input: f32) -> f32 {
        let out = self.line[self.at];
        let filtered = out * (1.0 - self.damping) + self.low_pass * self.damping;
        // Quiet tails are cut to zero (no slow denormal numbers).
        self.low_pass = if filtered.abs() < 1.0e-20 {
            0.0
        } else {
            filtered
        };
        self.line[self.at] = input * (1.0 - self.feedback) + self.low_pass * self.feedback;
        self.at = (self.at + 1) % self.line.len();
        out
    }
}

struct Allpass {
    line: Vec<f32>,
    at: usize,
}

impl Allpass {
    fn new(length: usize) -> Self {
        Allpass {
            line: vec![0.0; length.max(1)],
            at: 0,
        }
    }

    fn step(&mut self, input: f32) -> f32 {
        let held = self.line[self.at];
        let out = held - input;
        let stored = input + held * ALLPASS_FEEDBACK;
        self.line[self.at] = if stored.abs() < 1.0e-20 { 0.0 } else { stored };
        self.at = (self.at + 1) % self.line.len();
        out
    }
}

/// One ear's network.
struct Ear {
    combs: Vec<Comb>,
    allpasses: Vec<Allpass>,
}

impl Ear {
    fn new(rate: u32, extra: usize) -> Self {
        let scaled = |n: usize| (n + extra) * rate as usize / 44100;
        Ear {
            combs: COMB_LENGTHS.iter().map(|&n| Comb::new(scaled(n))).collect(),
            allpasses: ALLPASS_LENGTHS
                .iter()
                .map(|&n| Allpass::new(scaled(n)))
                .collect(),
        }
    }

    fn step(&mut self, input: f32) -> f32 {
        let sum: f32 = self.combs.iter_mut().map(|c| c.step(input)).sum();
        let mut out = sum / COMB_LENGTHS.len() as f32;
        for a in &mut self.allpasses {
            out = a.step(out);
        }
        out
    }
}

/// The reverb: sends go in one at a time, a stereo tail comes out at the same rate.
pub struct Reverb {
    rate: u32,
    ears: [Ear; 2],
    decay: f32,
}

impl Reverb {
    /// A silent reverb for `rate` Hz, with the room the race starts in.
    pub fn new(rate: u32) -> Self {
        let mut r = Reverb {
            rate,
            ears: [Ear::new(rate, 0), Ear::new(rate, SPREAD)],
            decay: 0.0,
        };
        r.set_room(&ROOMS[RACE_ROOM]);
        r
    }

    /// The room's decay and damping taken (what is in the lines goes on sounding).
    pub fn set_room(&mut self, room: &Room) {
        let decay = room.decay.clamp(0.1, 20.0);
        let damping = (room.damping.clamp(0.0, 1.0) * 0.7).min(0.7);
        for ear in &mut self.ears {
            for c in &mut ear.combs {
                // Falls by 60 dB (a factor of 1000) over `decay` seconds.
                let seconds = c.line.len() as f32 / self.rate as f32;
                let f = crate::math::portable::powf(10.0, -3.0 * seconds / decay);
                c.feedback = f.min(MOST_FEEDBACK);
                c.damping = damping;
            }
        }
        self.decay = decay;
    }

    /// The decay time in seconds of the room set.
    pub fn decay(&self) -> f32 {
        self.decay
    }

    /// One send in, the tail's left and right out.
    pub fn step(&mut self, send: f32) -> (f32, f32) {
        // A comb is normalised to a gain of 1.0 for a steady input, so a steady sound has a
        // tail of the same level as itself (before the damping and the send level).
        (self.ears[0].step(send), self.ears[1].step(send))
    }

    /// Seconds past the last send after which the tail is inaudible (-80 dB).
    pub fn tail_seconds(&self) -> f32 {
        self.decay * 4.0 / 3.0 + 0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 44100;

    fn response(room: &Room, seconds: f32) -> Vec<(f32, f32)> {
        let mut r = Reverb::new(RATE);
        r.set_room(room);
        (0..(seconds * RATE as f32) as usize)
            .map(|n| r.step(if n == 0 { 1.0 } else { 0.0 }))
            .collect()
    }

    fn energy(part: &[(f32, f32)]) -> f32 {
        part.iter().map(|(l, r)| l * l + r * r).sum()
    }

    #[test]
    fn every_room_decays_and_stays_bounded() {
        for (n, room) in ROOMS.iter().enumerate() {
            let out = response(room, 12.0);
            assert!(
                out.iter().all(|(l, r)| l.is_finite() && r.is_finite()),
                "room {n}"
            );
            let peak = out
                .iter()
                .map(|(l, r)| l.abs().max(r.abs()))
                .fold(0.0, f32::max);
            assert!(peak < 1.0, "room {n} peaks at {peak}");
            // The tail falls: the second after the room's decay time holds under a millionth of
            // the energy of the first second.
            let second = RATE as usize;
            let after = (room.decay.clamp(0.1, 20.0) * RATE as f32) as usize + second;
            let first = energy(&out[..second]);
            let later = energy(&out[after..after + second]);
            assert!(first > 0.0, "room {n} is silent");
            assert!(later < first * 1.0e-6, "room {n}: {later} against {first}");
        }
    }

    #[test]
    fn a_longer_decay_time_sounds_longer() {
        let short = response(&ROOMS[3], 3.0);
        let long = response(&ROOMS[8], 3.0);
        let late = RATE as usize * 2;
        assert!(energy(&long[late..]) > energy(&short[late..]) * 1000.0);
    }

    #[test]
    fn damping_takes_the_highs_out_of_the_tail() {
        let bright = Room {
            environment: 0,
            volume: 1.0,
            decay: 2.0,
            damping: 0.0,
        };
        let dark = Room {
            damping: 1.0,
            ..bright
        };
        // The energy of the sample-to-sample change over the energy: high for a bright tail.
        let roughness = |room: &Room| {
            let out = response(room, 1.5);
            // The late tail, where the damping has acted on every trip round the lines.
            let out = &out[RATE as usize / 2..];
            let diff: f32 = out.windows(2).map(|w| (w[1].0 - w[0].0).powi(2)).sum();
            diff / out.iter().map(|(l, _)| l * l).sum::<f32>()
        };
        let (d, b) = (roughness(&dark), roughness(&bright));
        assert!(d < b * 0.5, "{d} against {b}");
    }

    #[test]
    fn loud_steady_and_noisy_sends_stay_bounded() {
        let mut r = Reverb::new(RATE);
        r.set_room(&ROOMS[8]);
        let mut seed = 12345u32;
        let mut peak = 0f32;
        for n in 0..RATE as usize * 8 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = (seed >> 8) as f32 / (1 << 23) as f32 - 1.0;
            let send = if n % 2 == 0 { noise } else { 1.0 };
            let (l, rr) = r.step(send);
            assert!(l.is_finite() && rr.is_finite());
            peak = peak.max(l.abs()).max(rr.abs());
        }
        assert!(peak < 4.0, "peak {peak}");
    }

    #[test]
    fn the_send_follows_the_room_and_the_weight() {
        assert_eq!(send_level(&ROOMS[4], 10.0), 0.0);
        assert_eq!(send_level(&ROOMS[3], 0.0), 0.0);
        assert_eq!(send_level(&ROOMS[3], f32::NAN), 0.0);
        let near = send_level(&ROOMS[3], 1.0);
        let open = send_level(&ROOMS[3], 30.0);
        let inside = send_level(&ROOMS[3], 200.0);
        assert!(0.0 < near && near < open && open < inside && inside < WET_LEVEL);
        // Open country is barely heard, an enclosed place several times louder.
        assert!(open < WET_LEVEL * OPEN_SHARE);
        assert!(inside > open * 4.0);
    }

    /// The reverb's cost beside a sound: `cargo test -p rc2k_core --release reverb -- --nocapture
    /// --ignored` prints the time a second of sound takes.
    #[test]
    #[ignore = "a timing, not a check"]
    fn cost_of_a_second() {
        let mut r = Reverb::new(RATE);
        r.set_room(&ROOMS[8]);
        let start = std::time::Instant::now();
        let mut sink = 0f32;
        for n in 0..RATE as usize * 20 {
            let (l, rr) = r.step((n % 100) as f32 * 0.01);
            sink += l + rr;
        }
        let per_second = start.elapsed().as_secs_f64() / 20.0;
        println!(
            "reverb: {:.3} ms per second of sound ({sink})",
            per_second * 1000.0
        );
    }
}
