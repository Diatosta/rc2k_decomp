//! The game's numbers: its float constants (1.0, 65536.0, pi, the tenths, ...), named by value,
//! with three variables among them (the replay's mode, `in_race`, the grip change) kept apart.
//! The constants are a plain `Copy` value every reader holds by reference; the variables are
//! plain cells, a drawing store (`session::DrawShared`), written on the game's thread between
//! frames and read by the drawing parts through their `scene::workers::DrawReads`.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{ConstFloat, PlainDword, PlainFlag, PlainFloat};

/// The game's float constants (a plain value: nothing writes them).
#[derive(Clone, Copy)]
pub struct Numbers {
    /// 1609.3457 (`metres_in_mile`).
    pub metres_in_mile: ConstFloat,
    /// 0.01.
    pub hundredth_a: ConstFloat,
    /// 2.25.
    pub two_and_a_quarter: ConstFloat,
    /// 16.0.
    pub sixteen: ConstFloat,
    /// 20.0.
    pub twenty: ConstFloat,
    /// 100.0.
    pub hundred: ConstFloat,
    /// 1000.0.
    pub thousand: ConstFloat,
    /// 32.0.
    pub thirty_two: ConstFloat,
    /// 1.0.
    pub one: ConstFloat,
    /// 2.0.
    pub two: ConstFloat,
    /// 3.0.
    pub three: ConstFloat,
    /// 4.0.
    pub four: ConstFloat,
    /// 5.0.
    pub five: ConstFloat,
    /// 6.0.
    pub six: ConstFloat,
    /// 8.0.
    pub eight: ConstFloat,
    /// 10.0.
    pub ten: ConstFloat,
    /// 12.0.
    pub twelve: ConstFloat,
    /// 60.0.
    pub sixty: ConstFloat,
    /// 64.0.
    pub sixty_four: ConstFloat,
    /// 384.0.
    pub k384: ConstFloat,
    /// 800.0.
    pub k800: ConstFloat,
    /// 600.0.
    pub k600: ConstFloat,
    /// -1.0.
    pub minus_one: ConstFloat,
    /// 0.2.
    pub fifth: ConstFloat,
    /// 0.01.
    pub hundredth: ConstFloat,
    /// 0.0.
    pub zero: ConstFloat,
    /// 65536.0 (a 16.16 whole number's scale, a turn's units).
    pub k65536: ConstFloat,
    /// 1 / 65536.
    pub one_over_65536: ConstFloat,
    /// 0.25.
    pub quarter: ConstFloat,
    /// 1.5.
    pub one_and_a_half: ConstFloat,
    /// Pi.
    pub pi: ConstFloat,
    /// 1 / pi.
    pub one_over_pi: ConstFloat,
    /// 0.1.
    pub tenth: ConstFloat,
    /// 1 / (2 pi).
    pub one_over_two_pi: ConstFloat,
    /// 0.1, 0.2 .. 0.9.
    pub tenths: [ConstFloat; 9],
    /// 2.56.
    pub brake_scale: ConstFloat,
    /// 1.6093457.
    pub kilometres_in_mile: ConstFloat,
    /// 4096.0.
    pub k4096_a: ConstFloat,
    /// 4096.0.
    pub k4096_b: ConstFloat,
    /// 4096.0.
    pub k4096: ConstFloat,
    /// 256.0.
    pub k256: ConstFloat,
    /// 10000.0.
    pub k10000: ConstFloat,
    /// 0.5.
    pub half: ConstFloat,
    /// 98304.0.
    pub k98304: ConstFloat,
}

/// The three variables among the game's numbers, read by the drawing parts and written on the
/// game's thread between frames.
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct NumberVars {
    /// The replay's mode, 1 recording, 2 playing (`drive::keep_replay_frame`; the scene skips
    /// the tyre screen while 2).
    pub replay_mode: PlainDword,
    /// `in_race`.
    pub in_race: PlainFlag,
    /// The grip forces' sum's change, eased to 0 (`physics::drive_grip_forces`).
    pub grip_change: PlainFloat,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl NumberVars {
    pub const fn new() -> Self {
        Self::ZERO
    }
}

/// The constants.
impl Numbers {
    pub const fn new() -> Self {
        Self {
            zero: ConstFloat::new(0.0),
            metres_in_mile: ConstFloat::new(1609.3457),
            hundredth_a: ConstFloat::new(0.01),
            two_and_a_quarter: ConstFloat::new(2.25),
            sixteen: ConstFloat::new(16.0),
            twenty: ConstFloat::new(20.0),
            hundred: ConstFloat::new(100.0),
            thousand: ConstFloat::new(1000.0),
            thirty_two: ConstFloat::new(32.0),
            one: ConstFloat::new(1.0),
            two: ConstFloat::new(2.0),
            three: ConstFloat::new(3.0),
            four: ConstFloat::new(4.0),
            five: ConstFloat::new(5.0),
            six: ConstFloat::new(6.0),
            eight: ConstFloat::new(8.0),
            ten: ConstFloat::new(10.0),
            twelve: ConstFloat::new(12.0),
            sixty: ConstFloat::new(60.0),
            sixty_four: ConstFloat::new(64.0),
            k384: ConstFloat::new(384.0),
            k800: ConstFloat::new(800.0),
            k600: ConstFloat::new(600.0),
            minus_one: ConstFloat::new(-1.0),
            fifth: ConstFloat::new(0.2),
            hundredth: ConstFloat::new(0.01),
            k65536: ConstFloat::new(65536.0),
            one_over_65536: ConstFloat::new(0.000015258789),
            quarter: ConstFloat::new(0.25),
            one_and_a_half: ConstFloat::new(1.5),
            pi: ConstFloat::new(core::f32::consts::PI),
            one_over_pi: ConstFloat::new(0.31830987),
            tenth: ConstFloat::new(0.1),
            one_over_two_pi: ConstFloat::new(0.15915494),
            tenths: [
                ConstFloat::new(0.1),
                ConstFloat::new(0.2),
                ConstFloat::new(0.3),
                ConstFloat::new(0.4),
                ConstFloat::new(0.5),
                ConstFloat::new(0.6),
                ConstFloat::new(0.7),
                ConstFloat::new(0.8),
                ConstFloat::new(0.9),
            ],
            brake_scale: ConstFloat::new(2.56),
            kilometres_in_mile: ConstFloat::new(1.6093457),
            k4096_a: ConstFloat::new(4096.0),
            k4096_b: ConstFloat::new(4096.0),
            k4096: ConstFloat::new(4096.0),
            k256: ConstFloat::new(256.0),
            k10000: ConstFloat::new(10000.0),
            half: ConstFloat::new(0.5),
            k98304: ConstFloat::new(98304.0),
        }
    }
}
