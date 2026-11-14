//! The game's numbers: its float constants (1.0, 65536.0, pi, the tenths, ...), named by value,
//! with three variables among them (the replay's mode, `in_race`, the grip change). Read-only but
//! for those three and `physics::clamped_asin`'s turn of `one`. The fields keep a fixed layout.
//! Plain cells: a drawing store (`session::DrawShared`), written on the game's thread
//! between frames and read by the drawing parts through their `workers::DrawReads`.

use crate::cells::ZeroCells;
use crate::session::{PlainDword, PlainFlag, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct Numbers {
    /// The replay's mode, 1 recording, 2 playing (`drive::keep_replay_frame`; the scene skips
    /// the tyre screen while 2).
    pub replay_mode: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_4: [PlainDword; 2],
    /// `in_race`.
    pub in_race: PlainFlag,
    /// 1609.3457 (`metres_in_mile`).
    pub metres_in_mile: PlainFloat,
    /// 0.001.
    pub thousandth_a: PlainFloat,
    /// 100.0.
    pub hundred_a: PlainFloat,
    /// 0.01.
    pub hundredth_a: PlainFloat,
    /// 2.25.
    pub two_and_a_quarter: PlainFloat,
    /// -1.0.
    pub minus_one_a: PlainFloat,
    /// 16.0.
    pub sixteen: PlainFloat,
    /// 20.0.
    pub twenty: PlainFloat,
    /// 40.0.
    pub forty: PlainFloat,
    /// 100.0.
    pub hundred: PlainFloat,
    /// 1000.0.
    pub thousand: PlainFloat,
    /// 1024.0.
    pub k1024: PlainFloat,
    /// 3000.0.
    pub k3000: PlainFloat,
    /// 3100.0.
    pub k3100: PlainFloat,
    /// 25.0.
    pub twenty_five: PlainFloat,
    /// 24.0.
    pub twenty_four: PlainFloat,
    /// 32.0.
    pub thirty_two: PlainFloat,
    /// 50.0.
    pub fifty: PlainFloat,
    /// 1.0 (`physics::clamped_asin` negates it and back around its own use).
    pub one: PlainFloat,
    /// 2.0.
    pub two: PlainFloat,
    /// 3.0.
    pub three: PlainFloat,
    /// 4.0.
    pub four: PlainFloat,
    /// 5.0.
    pub five: PlainFloat,
    /// 6.0.
    pub six: PlainFloat,
    /// 8.0.
    pub eight: PlainFloat,
    /// 10.0.
    pub ten: PlainFloat,
    /// 12.0.
    pub twelve: PlainFloat,
    /// 60.0.
    pub sixty: PlainFloat,
    /// 64.0.
    pub sixty_four: PlainFloat,
    /// 128.0.
    pub k128: PlainFloat,
    /// 384.0.
    pub k384: PlainFloat,
    /// 800.0.
    pub k800: PlainFloat,
    /// 600.0.
    pub k600: PlainFloat,
    /// -1.0.
    pub minus_one: PlainFloat,
    /// 0.2.
    pub fifth: PlainFloat,
    /// 1.1.
    pub one_point_one: PlainFloat,
    /// 0.9.
    pub nine_tenths_a: PlainFloat,
    /// 0.1.
    pub tenth_a: PlainFloat,
    /// 0.99.
    pub point_99: PlainFloat,
    /// 0.01.
    pub hundredth: PlainFloat,
    /// 0.0.
    pub zero: PlainFloat,
    /// 65536.0.
    pub k65536_a: PlainFloat,
    /// 65536.0 (a 16.16 whole number's scale, a turn's units).
    pub k65536: PlainFloat,
    /// 16777216.0.
    pub k16777216: PlainFloat,
    /// 1 / 65536.
    pub one_over_65536: PlainFloat,
    /// 0.25.
    pub quarter: PlainFloat,
    /// 0.75.
    pub three_quarters: PlainFloat,
    /// 1.25.
    pub five_quarters: PlainFloat,
    /// 2.5.
    pub two_and_a_half: PlainFloat,
    /// 1.5.
    pub one_and_a_half: PlainFloat,
    /// 0.1.
    pub tenth_b: PlainFloat,
    /// Pi.
    pub pi: PlainFloat,
    /// Pi / 2.
    pub half_pi: PlainFloat,
    /// 0.785.
    pub point_785: PlainFloat,
    /// 1 / pi.
    pub one_over_pi: PlainFloat,
    /// 2 / pi.
    pub two_over_pi: PlainFloat,
    /// 0.1.
    pub tenth: PlainFloat,
    /// 1 / (2 pi).
    pub one_over_two_pi: PlainFloat,
    /// 0.9.
    pub nine_tenths: PlainFloat,
    /// 0.1, 0.2 .. 0.9.
    pub tenths: [PlainFloat; 9],
    /// 2.56.
    pub brake_scale: PlainFloat,
    /// 0.95.
    pub point_95: PlainFloat,
    /// 1.6093457.
    pub kilometres_in_mile: PlainFloat,
    /// The debug font's pixel size (1 at first; `debug::draw` writes it before each of the
    /// overlay's prints).
    pub debug_scale: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_130: [PlainDword; 2],
    /// 4096.0.
    pub k4096_a: PlainFloat,
    /// 262144.0.
    pub k262144: PlainFloat,
    /// 4096.0.
    pub k4096_b: PlainFloat,
    /// 16384.0.
    pub k16384: PlainFloat,
    /// 4096.0.
    pub k4096: PlainFloat,
    /// 256.0.
    pub k256: PlainFloat,
    /// 512.0.
    pub k512: PlainFloat,
    /// 10000.0.
    pub k10000: PlainFloat,
    /// 80.0.
    pub eighty: PlainFloat,
    /// 48.0.
    pub forty_eight: PlainFloat,
    /// 0.5.
    pub half: PlainFloat,
    /// 0.001.
    pub thousandth: PlainFloat,
    /// The grip forces' sum's change, eased to 0 (`physics::drive_grip_forces`; a
    /// variable).
    pub grip_change: PlainFloat,
    /// 98304.0.
    pub k98304: PlainFloat,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with its first values.
impl Numbers {
    pub const fn new() -> Self {
        Self {
            // Game addresses:
            unused_4: [PlainDword::new(0x7A7978), PlainDword::new(0)],
            metres_in_mile: PlainFloat::new(1609.3457),
            thousandth_a: PlainFloat::new(0.001),
            hundred_a: PlainFloat::new(100.0),
            hundredth_a: PlainFloat::new(0.01),
            two_and_a_quarter: PlainFloat::new(2.25),
            minus_one_a: PlainFloat::new(-1.0),
            sixteen: PlainFloat::new(16.0),
            twenty: PlainFloat::new(20.0),
            forty: PlainFloat::new(40.0),
            hundred: PlainFloat::new(100.0),
            thousand: PlainFloat::new(1000.0),
            k1024: PlainFloat::new(1024.0),
            k3000: PlainFloat::new(3000.0),
            k3100: PlainFloat::new(3100.0),
            twenty_five: PlainFloat::new(25.0),
            twenty_four: PlainFloat::new(24.0),
            thirty_two: PlainFloat::new(32.0),
            fifty: PlainFloat::new(50.0),
            one: PlainFloat::new(1.0),
            two: PlainFloat::new(2.0),
            three: PlainFloat::new(3.0),
            four: PlainFloat::new(4.0),
            five: PlainFloat::new(5.0),
            six: PlainFloat::new(6.0),
            eight: PlainFloat::new(8.0),
            ten: PlainFloat::new(10.0),
            twelve: PlainFloat::new(12.0),
            sixty: PlainFloat::new(60.0),
            sixty_four: PlainFloat::new(64.0),
            k128: PlainFloat::new(128.0),
            k384: PlainFloat::new(384.0),
            k800: PlainFloat::new(800.0),
            k600: PlainFloat::new(600.0),
            minus_one: PlainFloat::new(-1.0),
            fifth: PlainFloat::new(0.2),
            one_point_one: PlainFloat::new(1.1),
            nine_tenths_a: PlainFloat::new(0.9),
            tenth_a: PlainFloat::new(0.1),
            point_99: PlainFloat::new(0.99),
            hundredth: PlainFloat::new(0.01),
            k65536_a: PlainFloat::new(65536.0),
            k65536: PlainFloat::new(65536.0),
            k16777216: PlainFloat::new(16777216.0),
            one_over_65536: PlainFloat::new(0.000015258789),
            quarter: PlainFloat::new(0.25),
            three_quarters: PlainFloat::new(0.75),
            five_quarters: PlainFloat::new(1.25),
            two_and_a_half: PlainFloat::new(2.5),
            one_and_a_half: PlainFloat::new(1.5),
            tenth_b: PlainFloat::new(0.1),
            pi: PlainFloat::new(core::f32::consts::PI),
            half_pi: PlainFloat::new(1.5707964),
            point_785: PlainFloat::new(0.785),
            one_over_pi: PlainFloat::new(0.31830987),
            two_over_pi: PlainFloat::new(0.63661975),
            tenth: PlainFloat::new(0.1),
            one_over_two_pi: PlainFloat::new(0.15915494),
            nine_tenths: PlainFloat::new(0.9),
            tenths: [
                PlainFloat::new(0.1),
                PlainFloat::new(0.2),
                PlainFloat::new(0.3),
                PlainFloat::new(0.4),
                PlainFloat::new(0.5),
                PlainFloat::new(0.6),
                PlainFloat::new(0.7),
                PlainFloat::new(0.8),
                PlainFloat::new(0.9),
            ],
            brake_scale: PlainFloat::new(2.56),
            point_95: PlainFloat::new(0.95),
            kilometres_in_mile: PlainFloat::new(1.6093457),
            debug_scale: PlainDword::new(1),
            k4096_a: PlainFloat::new(4096.0),
            k262144: PlainFloat::new(262144.0),
            k4096_b: PlainFloat::new(4096.0),
            k16384: PlainFloat::new(16384.0),
            k4096: PlainFloat::new(4096.0),
            k256: PlainFloat::new(256.0),
            k512: PlainFloat::new(512.0),
            k10000: PlainFloat::new(10000.0),
            eighty: PlainFloat::new(80.0),
            forty_eight: PlainFloat::new(48.0),
            half: PlainFloat::new(0.5),
            thousandth: PlainFloat::new(0.001),
            k98304: PlainFloat::new(98304.0),
            ..ZeroCells::ZERO
        }
    }
}
