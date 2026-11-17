//! The race's frames drawn between its ticks. The simulation ticks a hundred times a second;
//! a screen that shows more frames than that would repeat a tick. With the smooth choice on
//! (`set_smooth`), the values of each car the drawing reads that move (its place, its turn,
//! its wheels' turns and heights, its camera) are taken after every tick, and a frame between
//! two ticks draws the blend of the last two (`rc2k_core::race::blend`), a tick behind the
//! simulation at most. The blocks hold the blend only while the frame is drawn
//! (`Blended`); nothing the simulation reads is changed, so races and replays play out tick
//! for tick as with the choice off.
//!
//! Frames not preceded by a tick step (the pause, a still of a replay) show the last tick
//! exactly. A car or camera that moved a long way in one tick (a reset, a camera cut) is not
//! blended across it.

use crate::memory::session::{read_all, write_all};
use crate::physics::car_store::Car;
use crate::race::blocks_store::BLOCK_COUNT;
use crate::race::owner::Race;
use rc2k_core::race::blend as core;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};

/// What moves with the car: its place and turn, the wheels.
#[derive(Clone, Copy, PartialEq)]
struct Body {
    position: [u32; 3],
    crash_matrix: [u32; 16],
    inverse_matrix: [u32; 16],
    wheel_turns: [u32; 2],
    squeeze: [u32; 4],
    spring_force: [u32; 4],
    ground_x: [u32; 4],
    ground_height: [u32; 4],
    ground_z: [u32; 4],
}

/// What moves with the car's camera.
#[derive(Clone, Copy, PartialEq)]
struct Camera {
    mode: u32,
    position: [u32; 3],
    /// Heading, pitch, roll turn and the three offset turns.
    angles: [u32; 6],
    zoom: u32,
    roll: u32,
}

#[derive(Clone, Copy, PartialEq)]
struct Snap {
    body: Body,
    camera: Camera,
}

fn read(car: &Car) -> Snap {
    Snap {
        body: Body {
            position: read_all(&car.position),
            crash_matrix: read_all(&car.crash_matrix),
            inverse_matrix: read_all(&car.car_inverse_matrix),
            wheel_turns: read_all(&car.driven_wheel_turns),
            squeeze: read_all(&car.wheel_squeeze),
            spring_force: read_all(&car.wheel_spring_force),
            ground_x: read_all(&car.wheel_ground_x),
            ground_height: read_all(&car.wheel_ground_height),
            ground_z: read_all(&car.wheel_ground_z),
        },
        camera: Camera {
            mode: car.camera_view.get(),
            position: read_all(&car.camera_position),
            angles: [
                car.camera_heading.get(),
                car.camera_pitch.get(),
                car.camera_roll_turn.get(),
                car.camera_offset_turns[0].get(),
                car.camera_offset_turns[1].get(),
                car.camera_offset_turns[2].get(),
            ],
            zoom: car.camera_zoom.bits(),
            roll: car.camera_roll.bits(),
        },
    }
}

fn write(car: &Car, snap: &Snap) {
    let (b, c) = (&snap.body, &snap.camera);
    write_all(&car.position, b.position);
    write_all(&car.crash_matrix, b.crash_matrix);
    write_all(&car.car_inverse_matrix, b.inverse_matrix);
    write_all(&car.driven_wheel_turns, b.wheel_turns);
    write_all(&car.wheel_squeeze, b.squeeze);
    write_all(&car.wheel_spring_force, b.spring_force);
    write_all(&car.wheel_ground_x, b.ground_x);
    write_all(&car.wheel_ground_height, b.ground_height);
    write_all(&car.wheel_ground_z, b.ground_z);
    write_all(&car.camera_position, c.position);
    car.camera_heading.set(c.angles[0]);
    car.camera_pitch.set(c.angles[1]);
    car.camera_roll_turn.set(c.angles[2]);
    for (cell, angle) in car.camera_offset_turns.iter().zip(&c.angles[3..]) {
        cell.set(*angle);
    }
    car.camera_zoom.set_bits(c.zoom);
    car.camera_roll.set_bits(c.roll);
}

fn words<const N: usize>(
    prev: &[u32; N],
    cur: &[u32; N],
    alpha: f32,
    f: fn(u32, u32, f32) -> u32,
) -> [u32; N] {
    std::array::from_fn(|k| f(prev[k], cur[k], alpha))
}

/// The snapshot between `prev` and `cur` at `alpha`.
fn blend(prev: &Snap, cur: &Snap, alpha: f32) -> Snap {
    let (p, c) = (&prev.body, &cur.body);
    let moved_far = p
        .position
        .iter()
        .zip(&c.position)
        .any(|(&a, &b)| !core::near(a, b, core::JUMP))
        || !core::matrices_near(&p.crash_matrix, &c.crash_matrix);
    let body = if moved_far {
        *c
    } else {
        Body {
            position: words(&p.position, &c.position, alpha, core::word),
            crash_matrix: words(&p.crash_matrix, &c.crash_matrix, alpha, core::float),
            inverse_matrix: words(&p.inverse_matrix, &c.inverse_matrix, alpha, core::float),
            wheel_turns: words(&p.wheel_turns, &c.wheel_turns, alpha, core::word),
            squeeze: words(&p.squeeze, &c.squeeze, alpha, core::float),
            spring_force: words(&p.spring_force, &c.spring_force, alpha, core::word),
            ground_x: words(&p.ground_x, &c.ground_x, alpha, core::word),
            ground_height: words(&p.ground_height, &c.ground_height, alpha, core::word),
            ground_z: words(&p.ground_z, &c.ground_z, alpha, core::word),
        }
    };
    let (p, c) = (&prev.camera, &cur.camera);
    let cut = p.mode != c.mode
        || p.position
            .iter()
            .zip(&c.position)
            .any(|(&a, &b)| !core::near(a, b, core::JUMP))
        || p.angles
            .iter()
            .zip(&c.angles)
            .any(|(&a, &b)| !core::near(a, b, core::JUMP_ANGLE));
    let camera = if cut {
        *c
    } else {
        Camera {
            mode: c.mode,
            position: words(&p.position, &c.position, alpha, core::word),
            angles: words(&p.angles, &c.angles, alpha, core::word),
            zoom: core::float(p.zoom, c.zoom, alpha),
            roll: core::float(p.roll, c.roll, alpha),
        }
    };
    Snap { body, camera }
}

/// A car's last two ticks.
#[derive(Clone, Copy, Default)]
struct Pair {
    prev: Option<Snap>,
    cur: Option<Snap>,
}

static PAIRS: Mutex<[Pair; BLOCK_COUNT]> = Mutex::new(
    [Pair {
        prev: None,
        cur: None,
    }; BLOCK_COUNT],
);

fn pairs() -> std::sync::MutexGuard<'static, [Pair; BLOCK_COUNT]> {
    PAIRS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The race's ticks begin: nothing is known of the ticks before.
pub fn reset() {
    *pairs() = [Pair::default(); BLOCK_COUNT];
    ALPHA.store(1.0f32.to_bits(), Relaxed);
}

/// A tick has run: each car's values kept beside the tick before.
pub fn note_tick(race: &Race) {
    let mut pairs = pairs();
    for (pair, car) in pairs.iter_mut().zip(&race.car_blocks().cars) {
        pair.prev = pair.cur;
        pair.cur = Some(read(car));
    }
}

/// Whether frames between ticks are blended (the advanced page's "smooth"); off, a frame shows
/// the last tick as it is.
static SMOOTH: AtomicBool = AtomicBool::new(true);

pub fn smooth_on() -> bool {
    SMOOTH.load(Relaxed)
}

pub fn set_smooth(on: bool) {
    SMOOTH.store(on, Relaxed);
}

/// A race's loop is running: the frames are blended ones (`blends_frames`).
static RACING: AtomicBool = AtomicBool::new(false);

/// The race's loop runs while this lives.
pub struct Racing;

impl Racing {
    pub fn begin() -> Self {
        reset();
        RACING.store(true, Relaxed);
        Racing
    }
}

impl Drop for Racing {
    fn drop(&mut self) {
        RACING.store(false, Relaxed);
    }
}

/// Whether the frames now shown are blended between the ticks: the smooth choice is on and a race
/// is running (the menus' frames are at most a hundred a second).
pub fn blends_frames() -> bool {
    smooth_on() && RACING.load(Relaxed)
}

/// Where the next frame is between the ticks (`rc2k_core::race::blend::alpha`, as float
/// bits); 1 (the last tick) unless a tick step has just set it.
static ALPHA: AtomicU32 = AtomicU32::new(0x3F80_0000);

/// The ticks of this turn are done (`drive::race_step`): the frame to draw is `rest`
/// milliseconds past the last of them; `ticking` false (the simulation paused) shows the last
/// tick.
pub fn ticks_done(rest: u32, ticking: bool) {
    // Scripted runs with a held clock (one frame, one tick) show the ticks as they are, so their
    // pictures do not depend on the clock's phase; `RC2K_BLEND_HELD` blends them anyway.
    #[cfg(feature = "remote")]
    if crate::remote::held_clock().is_some() && std::env::var_os("RC2K_BLEND_HELD").is_none() {
        return;
    }
    let alpha = if ticking { core::alpha(rest) } else { 1.0 };
    ALPHA.store(alpha.to_bits(), Relaxed);
}

/// The blocks holding the frame's blend while it is drawn; the ticks' values are put back when
/// it is dropped.
pub struct Blended<'a> {
    race: &'a Race,
    /// Each blended car's number, its tick's values and the blend.
    saved: Vec<(usize, Snap, Snap)>,
}

impl<'a> Blended<'a> {
    /// Takes the frame's place between the ticks (it is the last tick again for the next frame)
    /// and puts the blend in the blocks of the cars that have two ticks to blend and whose
    /// blocks are still as the last tick left them.
    pub fn new(race: &'a Race) -> Self {
        let alpha = f32::from_bits(ALPHA.swap(0x3F80_0000, Relaxed));
        let mut saved = Vec::new();
        if smooth_on() && alpha < 1.0 {
            let pairs = pairs();
            for (n, (pair, car)) in pairs.iter().zip(&race.car_blocks().cars).enumerate() {
                let (Some(prev), Some(cur)) = (&pair.prev, &pair.cur) else {
                    continue;
                };
                let now = read(car);
                if now != *cur {
                    continue;
                }
                let blended = blend(prev, cur, alpha);
                write(car, &blended);
                saved.push((n, now, blended));
            }
        }
        Blended { race, saved }
    }
}

impl Blended<'_> {
    fn restore(&mut self) {
        for (n, snap, _) in self.saved.drain(..) {
            write(&self.race.car_blocks().cars[n], &snap);
        }
    }

    /// `f` run with the ticks' values in the blocks, for what keeps state from one frame to the
    /// next that depends on the places it saw (the maps' scroll).
    pub fn at_the_tick<T>(&self, f: impl FnOnce() -> T) -> T {
        for (n, tick, _) in &self.saved {
            write(&self.race.car_blocks().cars[*n], tick);
        }
        let out = f();
        for (n, _, blended) in &self.saved {
            write(&self.race.car_blocks().cars[*n], blended);
        }
        out
    }

    /// The frame is drawn: the ticks' values are put back, and the parts' matrices the drawing
    /// kept in the blocks (which the simulation's sound reads) are made again from them, as an
    /// unblended frame leaves them.
    pub fn finish(
        mut self,
        scene: &crate::scene::owner::Scene,
    ) -> Result<(), crate::errors::Refused> {
        if self.saved.is_empty() {
            return Ok(());
        }
        self.restore();
        let _held = crate::race::frame_view::Held::take(self.race);
        crate::race::car_parts::ready_car_parts(self.race, scene)
    }
}

impl Drop for Blended<'_> {
    fn drop(&mut self) {
        self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(x: u32) -> Snap {
        let mut s = read(&Car::new());
        s.body.position[0] = x;
        s.camera.position[0] = x;
        s.camera.angles[0] = x << 10;
        s
    }

    #[test]
    fn a_blend_lies_between_the_ticks() {
        let (a, b) = (snap(1000), snap(2000));
        let half = blend(&a, &b, 0.5);
        assert_eq!(half.body.position[0], 1500);
        assert_eq!(half.camera.position[0], 1500);
        assert!(blend(&a, &b, 1.0) == b);
        assert!(blend(&a, &b, 0.0) == a);
    }

    #[test]
    fn a_reset_or_a_cut_shows_the_tick() {
        let (a, b) = (snap(1000), snap(1000 + (20 << 16)));
        assert!(blend(&a, &b, 0.5) == b);
        let mut c = snap(1000);
        c.camera.mode = 5;
        assert!(blend(&a, &c, 0.5).camera == c.camera);
    }

    #[test]
    fn the_blocks_get_the_blend_for_a_frame_only() {
        let race = crate::test_support::own::race();
        let _lock = crate::test_support::test_lock();
        let car = &race.car_blocks().cars[0];
        let (a, b) = (snap(1000), snap(2000));
        write(car, &a);
        note_tick(race);
        write(car, &b);
        note_tick(race);
        ALPHA.store(0.5f32.to_bits(), Relaxed);
        {
            let _blended = Blended::new(race);
            assert_eq!(car.position[0].get(), 1500);
        }
        assert_eq!(car.position[0].get(), 2000);
        // The frame after is the tick itself.
        {
            let _blended = Blended::new(race);
            assert_eq!(car.position[0].get(), 2000);
        }
        reset();
    }
}
