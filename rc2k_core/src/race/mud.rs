//! The mud on the car's textures: places on a 256 x 256 square, the mud map's strengths
//! (128 x 128 bytes a texture), the throws and the splashes (converted 2026-10-21 from
//! `race::mud_pixel` 0x44A58A, `mud_texel` 0x44A549, `put_mud_texel` 0x44A5A9,
//! `muddy_texel` 0x44A2E1, `throw_mud_texel` 0x44A2A2 and `splash_mud` 0x44A35F; the
//! turns, the colours and the file converted 2026-10-21 from `race::mud_frame` 0x44A054,
//! `mud_texture_turn` 0x44A087, `sample_mud_colours` 0x44A0D6 and `load_mud` 0x449D95;
//! the tint converted 2026-10-21 from `race::tint_to_mud_colour` 0x44A441).

/// The mud textures (`dword_623D5C`: 0 .. 3).
pub const MOST_TEXTURE: u32 = 3;
/// One texture's map: 128 x 128 strengths.
pub const MAP_BYTES: u32 = 0x4000;

/// `race::mud_texel` / `put_mud_texel`: a place on the 256 x 256 square on the texture,
/// halved `halvings` times (`dword_5CEEE0`: 0 none, 1 once, more twice; x then y each
/// time), or None when an odd place would be halved.
pub fn texel_place(x: u32, y: u32, halvings: u32) -> Option<(u32, u32)> {
    let times = halvings.min(2);
    let (mut x, mut y) = (x, y);
    for _ in 0..times {
        if x & 1 != 0 {
            return None;
        }
        x >>= 1;
        if y & 1 != 0 {
            return None;
        }
        y >>= 1;
    }
    Some((x, y))
}

/// `race::mud_pixel` / `put_mud_texel`: whether (x, y) is inside a surface of `width` x
/// `height` (unsigned).
pub fn inside(x: u32, y: u32, width: u32, height: u32) -> bool {
    x < width && y < height
}

/// `race::muddy_texel`: the place of a shuffled word (x its low byte, y its high one).
pub fn shuffled_place(word: u16) -> (u32, u32) {
    ((word & 0xFF) as u32, (word >> 8) as u32)
}

/// The strength's place in the mud map for texture `n` at (x, y): (x / 2, y / 2) of its
/// 128 x 128.
pub fn strength_index(n: u32, x: u32, y: u32) -> u32 {
    (n * MAP_BYTES)
        .wrapping_add(x >> 1)
        .wrapping_add(strength_row(y))
}

/// The row's part of `strength_index`: (y / 2) x 128 (what the address's making leaves
/// in eax).
pub fn strength_row(y: u32) -> u32 {
    (y >> 1) << 7
}

/// Whether a texel (its components) takes mud: its top bit.
pub fn muddy(texel: u32) -> bool {
    texel & 0x8000_0000 != 0
}

/// The top byte a texel gets when mud is put on it.
pub const MUD_TOP: u32 = 0xFF00_0000;

/// `race::muddy_texel`: how far a texel is tinted toward the mud's colour: (strength + 1)
/// / 2.
pub fn tint_amount(strength: u32) -> u32 {
    (strength + 1) >> 1
}

/// The throws a texture takes (`race::throw_mud_texel`: its count `dword_623D80[n]`).
pub const LAST_THROWN: u32 = 0x6_0000;

/// `race::throw_mud_texel`: the extra throw's sum (`dword_623D30`) on by 0x10000000, and
/// whether it carried (every 16th: a texel at a random number too).
pub fn extra_throw(sum: u32) -> (u32, bool) {
    sum.overflowing_add(0x1000_0000)
}

/// The weakest strength that splashes (`race::splash_mud`).
pub const WEAKEST: u32 = 0x20;

/// `race::splash_mud`: the place from a random number (x its low byte, y the next).
pub fn splash_place(random: u32) -> (u32, u32) {
    let r = random & 0xFFFF;
    (r & 0xFF, (r >> 8) & 0xFF)
}

/// `race::splash_mud`: whether a splash lands at `strength`: not below `WEAKEST`, and the
/// random byte (`roll`, drawn only then) not above it.
pub fn splash_lands(strength: u32, roll: impl FnOnce() -> u32) -> bool {
    strength >= WEAKEST && roll() & 0xFF <= strength
}

/// `race::splash_mud`: the pattern's place in `g_mudColours` (16 of 4 x 4 dwords) from a
/// random number.
pub fn splash_pattern(random: u32) -> u32 {
    (random & 0xF) << 6
}

/// The splash's side (4 x 4 texels).
pub const SPLASH_SIDE: u32 = 4;

/// The frames between two mud textures' turns.
pub const TURN_FRAMES: u32 = 0xE;

/// The frame count (`dword_623D58`) one on: the count kept and whether a texture's turn
/// is due (at `TURN_FRAMES` or past, the count back to 0).
pub fn count_frame(frames: u32) -> (u32, bool) {
    let n = frames.wrapping_add(1);
    if n < TURN_FRAMES {
        (n, false)
    } else {
        (0, true)
    }
}

/// The texture worked on, held to 0 .. 3 (`and 3`), and the next after it.
pub fn texture_turn(texture: u32) -> u32 {
    texture & MOST_TEXTURE
}

/// The texture after `texture`, held to 0 .. 3.
pub fn next_texture(texture: u32) -> u32 {
    texture.wrapping_add(1) & MOST_TEXTURE
}

/// The mud's colour packed (`dword_623D34`) from its red, green and blue dwords: red
/// << 16 with its low half replaced by green's and blue's low bytes.
pub fn packed_colour([red, green, blue]: [u32; 3]) -> u32 {
    (red << 16) & 0xFFFF_0000 | (green & 0xFF) << 8 | (blue & 0xFF)
}

/// The colours sampled from the mud's picture: 256, colour n's at (0x89 + (n & 3) +
/// (n >> 5) x 4, 0x68 + ((n & 0x1C) >> 2)).
pub fn sample_place(n: u32) -> (u32, u32) {
    (0x89 + (n & 3) + (n >> 5) * 4, 0x68 + ((n & 0x1C) >> 2))
}

/// The colours sampled.
pub const SAMPLES: u32 = 0x100;

pub fn file_digits(number: u32) -> (u32, u32) {
    let digit = |v: u32| v & !0xFF | (v as u8).wrapping_add(b'0') as u32;
    (digit(number / 10), digit(number % 10))
}

/// A colour's tint toward the mud's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tint {
    /// The colour (its top byte kept).
    pub colour: u32,
    /// The high dword of the last byte's product (edx).
    pub high: u32,
}

/// `race::tint_to_mud_colour`: each of the colour's three low bytes brought toward the
/// mud's (red, green, blue: whole dwords; blue for the lowest byte) by `amount` of 0x100
/// (`gfx::fade_colour`'s step).
pub fn tint(colour: u32, mud: [u32; 3], amount: u32) -> Tint {
    use crate::gfx::pixels::fade_step;
    let [b0, b1, b2, b3] = colour.to_le_bytes();
    let [red, green, blue] = mud;
    let (r0, _) = fade_step(b0.into(), blue, amount);
    let (r1, _) = fade_step(b1.into(), green, amount);
    let (r2, high) = fade_step(b2.into(), red, amount);
    Tint {
        colour: u32::from_le_bytes([r0 as u8, r1 as u8, r2 as u8, b3]),
        high,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places() {
        assert_eq!(texel_place(3, 5, 0), Some((3, 5)));
        assert_eq!(texel_place(4, 6, 1), Some((2, 3)));
        assert_eq!(texel_place(4, 6, 2), None);
        assert_eq!(texel_place(8, 12, 7), Some((2, 3)));
        assert_eq!(texel_place(3, 4, 1), None);
        assert!(inside(0, 0, 1, 1) && !inside(1, 0, 1, 1) && !inside(0, u32::MAX, 9, 9));
        assert_eq!(shuffled_place(0x1234), (0x34, 0x12));
        assert_eq!(strength_index(2, 5, 7), 0x8000 + 2 + 3 * 128);
        assert_eq!(strength_row(0xFF), 127 * 128);
        assert!(muddy(0x8000_0000) && !muddy(0x7FFF_FFFF));
        assert_eq!(tint_amount(0xFF), 0x80);
    }

    #[test]
    fn throws_and_splashes() {
        assert_eq!(extra_throw(0xF000_0000), (0, true));
        assert_eq!(extra_throw(0), (0x1000_0000, false));
        assert_eq!(splash_place(0xABCD_1234), (0x34, 0x12));
        assert!(!splash_lands(0x1F, || unreachable!()));
        assert!(splash_lands(0x20, || 0x120));
        assert!(!splash_lands(0x20, || 0x21));
        assert_eq!(splash_pattern(0x1F), 0xF << 6);
    }

    #[test]
    fn turns_and_colours() {
        assert_eq!(count_frame(0xC), (0xD, false));
        assert_eq!(count_frame(0xD), (0, true));
        assert_eq!(count_frame(0x20), (0, true));
        assert_eq!((texture_turn(5), next_texture(3)), (1, 0));
        assert_eq!(packed_colour([0x1234_5678, 0x1AB, 0x2CD]), 0x5678_ABCD);
        assert_eq!(sample_place(0), (0x89, 0x68));
        assert_eq!(sample_place(0xFF), (0x89 + 3 + 28, 0x68 + 7));
        assert_eq!(file_digits(37), (b'3' as u32, b'7' as u32));
        assert_eq!(
            file_digits(0x1000),
            (0x100 | (0x99u8.wrapping_add(b'0')) as u32, b'6' as u32)
        );
    }

    #[test]
    fn tints() {
        assert_eq!(tint(0xAA10_2030, [0x80, 0x40, 0x20], 0).colour, 0xAA10_2030);
        assert_eq!(
            tint(0xAA10_2030, [0x80, 0x40, 0x20], 0x100).colour,
            0xAA80_4020
        );
        assert_eq!(
            tint(0x0000_0000, [0x200, 0x200, 0x200], 0x100).colour,
            0x00FF_FFFF
        );
    }
}
