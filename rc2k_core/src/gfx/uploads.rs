//! A picture put into a system texture's levels, its sizes.
//!
//! Sizes are counted in halvings from 256 by the game's table (five sizes). The picture
//! must be as many halvings from the texture across as down; it is halved that many times
//! for the first level and once more for each level after.

use crate::Invalid;

/// The most surfaces (levels) a record has.
pub const MOST_LEVELS: u32 = 9;

/// Which size is not in the table, or that the two differ: the fatal error's place in the
/// caller's five codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Apart {
    PictureWidth = 0,
    TextureWidth = 1,
    PictureHeight = 2,
    TextureHeight = 3,
    NotSquare = 4,
}

/// How many halvings the picture (`width` x `height`) is from the texture, the same across and
/// down (each difference not below 0). `halvings` is the table's lookup, asked in this order: the
/// picture's width, the texture's, the picture's height, the texture's.
pub fn halvings_apart(
    halvings: impl Fn(u32) -> Option<u32>,
    width: u32,
    height: u32,
    texture: impl Fn() -> (u32, u32),
) -> Result<u32, Apart> {
    let across = halvings(width).ok_or(Apart::PictureWidth)?;
    let wide = halvings(texture().0).ok_or(Apart::TextureWidth)?;
    let across = (wide as i32 - across as i32).max(0) as u32;
    let down = halvings(height).ok_or(Apart::PictureHeight)?;
    let high = halvings(texture().1).ok_or(Apart::TextureHeight)?;
    let down = (high as i32 - down as i32).max(0) as u32;
    if across != down {
        return Err(Apart::NotSquare);
    }
    Ok(across)
}

/// The levels filled for a record's count (at least one). Refuses more than the record's
/// surfaces.
pub fn levels(count: u32) -> Result<u32, Invalid> {
    let levels = count.max(1);
    if levels > MOST_LEVELS {
        return Err(Invalid("more levels than the record has surfaces"));
    }
    Ok(levels)
}

/// The halvings before level `level` is filled: `apart` for the first, one for each after.
pub fn halvings_before(level: u32, apart: u32) -> u32 {
    if level == 0 { apart } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(size: u32) -> Option<u32> {
        [256, 128, 64, 32, 16]
            .iter()
            .position(|&s| s == size)
            .map(|n| n as u32)
    }

    #[test]
    fn sizes() {
        assert_eq!(halvings_apart(table, 64, 64, || (256, 256)), Ok(0));
        assert_eq!(halvings_apart(table, 256, 128, || (64, 32)), Ok(2));
        assert_eq!(
            halvings_apart(table, 256, 256, || (64, 32)),
            Err(Apart::NotSquare)
        );
        assert_eq!(
            halvings_apart(table, 8, 256, || unreachable!()),
            Err(Apart::PictureWidth)
        );
        assert_eq!(
            halvings_apart(table, 16, 100, || (16, 16)),
            Err(Apart::PictureHeight)
        );
        assert_eq!(levels(0), Ok(1));
        assert!(levels(10).is_err());
        assert_eq!((halvings_before(0, 3), halvings_before(2, 3)), (3, 1));
    }
}
