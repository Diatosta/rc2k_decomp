//! The game's numbers (IDA 0x5C5CE0 .. 0x5C5E50; phase 7, typed): its float constants (1.0,
//! 65536.0, pi, the tenths, ...), named by value, with three variables among them (the
//! replay's mode, `g_inRace`, the grip change). Read-only but for those three and
//! `physics::clamped_asin`'s turn of `one`. `#[repr(C)]` with the image's layout, over the
//! race block's bytes (`NUMBERS`), as `CAR`: the code reaching them by address reads the
//! same bytes.

use crate::session::{Dword, Flag, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Numbers {
    /// +0x0 (0x5C5CE0): `dword_5C5CE0`: the replay's mode, 1 recording, 2 playing (`drive::keep_replay_frame`; the scene skips the tyre screen while 2).
    pub replay_mode: Dword,
    /// +0x4 (0x5C5CE4): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unknown_4: [Dword; 2],
    /// +0xc (0x5C5CEC): `g_inRace`.
    pub in_race: Flag,
    /// +0x10 (0x5C5CF0): 1609.3457 (`g_metresInMile`).
    pub metres_in_mile: Float,
    /// +0x14 (0x5C5CF4): 0.001.
    pub thousandth_a: Float,
    /// +0x18 (0x5C5CF8): 100.0.
    pub hundred_a: Float,
    /// +0x1c (0x5C5CFC): 0.01.
    pub hundredth_a: Float,
    /// +0x20 (0x5C5D00): 2.25.
    pub two_and_a_quarter: Float,
    /// +0x24 (0x5C5D04): -1.0.
    pub minus_one_a: Float,
    /// +0x28 (0x5C5D08): 16.0.
    pub sixteen: Float,
    /// +0x2c (0x5C5D0C): 20.0.
    pub twenty: Float,
    /// +0x30 (0x5C5D10): 40.0.
    pub forty: Float,
    /// +0x34 (0x5C5D14): 100.0.
    pub hundred: Float,
    /// +0x38 (0x5C5D18): 1000.0.
    pub thousand: Float,
    /// +0x3c (0x5C5D1C): 1024.0.
    pub k1024: Float,
    /// +0x40 (0x5C5D20): 3000.0.
    pub k3000: Float,
    /// +0x44 (0x5C5D24): 3100.0.
    pub k3100: Float,
    /// +0x48 (0x5C5D28): 25.0.
    pub twenty_five: Float,
    /// +0x4c (0x5C5D2C): 24.0.
    pub twenty_four: Float,
    /// +0x50 (0x5C5D30): 32.0.
    pub thirty_two: Float,
    /// +0x54 (0x5C5D34): 50.0.
    pub fifty: Float,
    /// +0x58 (0x5C5D38): 1.0 (`physics::clamped_asin` negates it and back around its own use).
    pub one: Float,
    /// +0x5c (0x5C5D3C): 2.0.
    pub two: Float,
    /// +0x60 (0x5C5D40): 3.0.
    pub three: Float,
    /// +0x64 (0x5C5D44): 4.0.
    pub four: Float,
    /// +0x68 (0x5C5D48): 5.0.
    pub five: Float,
    /// +0x6c (0x5C5D4C): 6.0.
    pub six: Float,
    /// +0x70 (0x5C5D50): 8.0.
    pub eight: Float,
    /// +0x74 (0x5C5D54): 10.0.
    pub ten: Float,
    /// +0x78 (0x5C5D58): 12.0.
    pub twelve: Float,
    /// +0x7c (0x5C5D5C): 60.0.
    pub sixty: Float,
    /// +0x80 (0x5C5D60): 64.0.
    pub sixty_four: Float,
    /// +0x84 (0x5C5D64): 128.0.
    pub k128: Float,
    /// +0x88 (0x5C5D68): 384.0.
    pub k384: Float,
    /// +0x8c (0x5C5D6C): 800.0.
    pub k800: Float,
    /// +0x90 (0x5C5D70): 600.0.
    pub k600: Float,
    /// +0x94 (0x5C5D74): -1.0.
    pub minus_one: Float,
    /// +0x98 (0x5C5D78): 0.2.
    pub fifth: Float,
    /// +0x9c (0x5C5D7C): 1.1.
    pub one_point_one: Float,
    /// +0xa0 (0x5C5D80): 0.9.
    pub nine_tenths_a: Float,
    /// +0xa4 (0x5C5D84): 0.1.
    pub tenth_a: Float,
    /// +0xa8 (0x5C5D88): 0.99.
    pub point_99: Float,
    /// +0xac (0x5C5D8C): 0.01.
    pub hundredth: Float,
    /// +0xb0 (0x5C5D90): 0.0.
    pub zero: Float,
    /// +0xb4 (0x5C5D94): 65536.0.
    pub k65536_a: Float,
    /// +0xb8 (0x5C5D98): 65536.0 (a 16.16 whole number's scale, a turn's units).
    pub k65536: Float,
    /// +0xbc (0x5C5D9C): 16777216.0.
    pub k16777216: Float,
    /// +0xc0 (0x5C5DA0): 1 / 65536.
    pub one_over_65536: Float,
    /// +0xc4 (0x5C5DA4): 0.25.
    pub quarter: Float,
    /// +0xc8 (0x5C5DA8): 0.75.
    pub three_quarters: Float,
    /// +0xcc (0x5C5DAC): 1.25.
    pub five_quarters: Float,
    /// +0xd0 (0x5C5DB0): 2.5.
    pub two_and_a_half: Float,
    /// +0xd4 (0x5C5DB4): 1.5.
    pub one_and_a_half: Float,
    /// +0xd8 (0x5C5DB8): 0.1.
    pub tenth_b: Float,
    /// +0xdc (0x5C5DBC): pi.
    pub pi: Float,
    /// +0xe0 (0x5C5DC0): pi / 2.
    pub half_pi: Float,
    /// +0xe4 (0x5C5DC4): 0.785.
    pub point_785: Float,
    /// +0xe8 (0x5C5DC8): 1 / pi.
    pub one_over_pi: Float,
    /// +0xec (0x5C5DCC): 2 / pi.
    pub two_over_pi: Float,
    /// +0xf0 (0x5C5DD0): 0.1.
    pub tenth: Float,
    /// +0xf4 (0x5C5DD4): 1 / (2 pi).
    pub one_over_two_pi: Float,
    /// +0xf8 (0x5C5DD8): 0.9.
    pub nine_tenths: Float,
    /// +0xfc (0x5C5DDC): 0.1, 0.2 .. 0.9.
    pub tenths: [Float; 9],
    /// +0x120 (0x5C5E00): 2.56.
    pub brake_scale: Float,
    /// +0x124 (0x5C5E04): 0.95.
    pub point_95: Float,
    /// +0x128 (0x5C5E08): 1.6093457.
    pub kilometres_in_mile: Float,
    /// +0x12c (0x5C5E0C): `g_debugScale`: the debug font's pixel size (1 in the image;
    /// `debug::draw`, written before each of the overlay's prints).
    pub debug_scale: Dword,
    /// +0x130 (0x5C5E10): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unknown_130: [Dword; 2],
    /// +0x138 (0x5C5E18): 4096.0.
    pub k4096_a: Float,
    /// +0x13c (0x5C5E1C): 262144.0.
    pub k262144: Float,
    /// +0x140 (0x5C5E20): 4096.0.
    pub k4096_b: Float,
    /// +0x144 (0x5C5E24): 16384.0.
    pub k16384: Float,
    /// +0x148 (0x5C5E28): 4096.0.
    pub k4096: Float,
    /// +0x14c (0x5C5E2C): 256.0.
    pub k256: Float,
    /// +0x150 (0x5C5E30): 512.0.
    pub k512: Float,
    /// +0x154 (0x5C5E34): 10000.0.
    pub k10000: Float,
    /// +0x158 (0x5C5E38): 80.0.
    pub eighty: Float,
    /// +0x15c (0x5C5E3C): 48.0.
    pub forty_eight: Float,
    /// +0x160 (0x5C5E40): 0.5.
    pub half: Float,
    /// +0x164 (0x5C5E44): 0.001.
    pub thousandth: Float,
    /// +0x168 (0x5C5E48): the grip forces' sum's change, eased to 0 (`physics::drive_grip_forces`; a variable).
    pub grip_change: Float,
    /// +0x16c (0x5C5E4C): 98304.0.
    pub k98304: Float,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a field of `race::owner::Race::course`, formerly a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl Numbers {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x5C5CE0, the first byte of the piece.
pub const START: usize = 0x5C5CE0;
/// IDA 0x5C5E50, the end of the piece.
pub const END: usize = 0x5C5E50;

const _: () = assert!(END - START == core::mem::size_of::<Numbers>());

/// Where IDA 0x5C5CE0 is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &crate::race::owner::RACE.course.numbers,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::race::owner::RACE.course.numbers,
            START,
            END,
        );
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(Numbers, replay_mode), 0x5C5CE0 - START);
        assert_eq!(offset_of!(Numbers, two_and_a_quarter), 0x5C5D00 - START);
        assert_eq!(offset_of!(Numbers, thousand), 0x5C5D18 - START);
        assert_eq!(offset_of!(Numbers, thirty_two), 0x5C5D30 - START);
        assert_eq!(offset_of!(Numbers, five), 0x5C5D48 - START);
        assert_eq!(offset_of!(Numbers, sixty_four), 0x5C5D60 - START);
        assert_eq!(offset_of!(Numbers, fifth), 0x5C5D78 - START);
        assert_eq!(offset_of!(Numbers, zero), 0x5C5D90 - START);
        assert_eq!(offset_of!(Numbers, three_quarters), 0x5C5DA8 - START);
        assert_eq!(offset_of!(Numbers, half_pi), 0x5C5DC0 - START);
        assert_eq!(offset_of!(Numbers, nine_tenths), 0x5C5DD8 - START);
        assert_eq!(offset_of!(Numbers, k262144), 0x5C5E1C - START);
        assert_eq!(offset_of!(Numbers, k10000), 0x5C5E34 - START);
        assert_eq!(offset_of!(Numbers, k98304), 0x5C5E4C - START);
    }
}
